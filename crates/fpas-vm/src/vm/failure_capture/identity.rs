//! Diagnostic-owned identities, secondary relationships, and snapshot lifetimes.

use super::{CaptureState, FailureDiagnostic, FailureSnapshot, Lease};
use crate::vm::{VmError, worker::Worker};
use std::sync::{Arc, Mutex, OnceLock, Weak, atomic::Ordering};

/// Shared capture owner; the draft freezes only when the terminal observer receives it.
pub(super) struct Candidate {
    /// Mutable only while actual hosted continuations are still propagating the error.
    pub(super) draft: Mutex<Option<FailureSnapshot>>,
    frozen: OnceLock<Arc<FailureSnapshot>>,
    _lease: Option<Lease>,
}

impl Candidate {
    /// Detach the completed draft without copying values or their budget lease.
    pub(super) fn freeze(&self) -> Arc<FailureSnapshot> {
        Arc::clone(self.frozen.get_or_init(|| {
            let mut draft = self.draft.lock().unwrap_or_else(|p| p.into_inner());
            Arc::new(
                draft
                    .take()
                    .unwrap_or_else(|| unreachable!("a candidate freezes only once")),
            )
        }))
    }
}

#[derive(Clone)]
/// Relationships attached to this diagnostic clone, sharing the original cause.
pub(super) struct Context {
    /// VM that allocated this identity.
    pub(super) owner: Weak<CaptureState>,
    /// Original pre-unwind state, shared across propagation.
    pub(super) cause: Arc<Candidate>,
    /// Secondary errors in the runtime's attachment order.
    pub(super) related: Option<Arc<Related>>,
    /// Number of retained relationships in the persistent attachment chain.
    pub(super) related_count: usize,
    /// Known secondary errors omitted by a storage or relationship bound.
    pub(super) related_omitted: usize,
}

/// A bounded diagnostic and its explicitly identified cause.
pub(super) struct Related {
    /// Previous attachment, so extending a clone never copies an entire collection.
    pub(super) previous: Option<Arc<Related>>,
    /// Shared original identity and any already attached secondary relationships.
    pub(super) context: Context,
    /// Charged presentation data; no unbounded diagnostic string clone is retained.
    pub(super) diagnostic: Arc<FailureDiagnostic>,
    _lease: Lease,
}

impl CaptureState {
    /// Preserve a context from this VM or create an explicit new fault identity.
    pub(super) fn ensure(self: &Arc<Self>, error: &mut VmError, worker: Option<&Worker>) {
        if error
            .attachment::<Context>()
            .is_some_and(|context| context.owner.ptr_eq(&Arc::downgrade(self)))
        {
            return;
        }
        let lease = self.reserve_model(
            std::mem::size_of::<Candidate>()
                + std::mem::size_of::<Context>()
                + std::mem::size_of::<FailureSnapshot>()
                + 32,
        );
        let mut snapshot = FailureSnapshot {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            task_id: worker.map(|worker| worker.task_id),
            frames: Vec::new(),
            total_frames: None,
            reason: Some("no_live_frame"),
            lease: Lease::default(),
        };
        if lease.is_none() {
            snapshot.reason = Some("retained_byte_limit");
        } else if let Some(worker) = worker {
            self.capture(worker, &mut snapshot);
        }
        error.set_attachment(Context {
            owner: Arc::downgrade(self),
            cause: Arc::new(Candidate {
                draft: Mutex::new(Some(snapshot)),
                frozen: OnceLock::new(),
                _lease: lease,
            }),
            related: None,
            related_count: 0,
            related_omitted: 0,
        });
    }
}

/// Attach relationships at the runtime's selection point, without interpreting help text.
pub(in crate::vm) fn attach_related(primary: &mut VmError, further: &[VmError]) {
    let Some(mut context) = primary.attachment::<Context>().cloned() else {
        return;
    };
    let Some(state) = context.owner.upgrade() else {
        return;
    };
    for error in further {
        if context.related_count >= super::MAX_RELATED {
            context.related_omitted = context.related_omitted.saturating_add(1);
            continue;
        }
        let Some(lease) = state.reserve_model(
            std::mem::size_of::<Related>()
                + std::mem::size_of::<FailureDiagnostic>()
                + std::mem::size_of::<fpas_diagnostics::Diagnostic>()
                + 48,
        ) else {
            context.related_omitted = context.related_omitted.saturating_add(1);
            continue;
        };
        let diagnostic = state.presentation(error);
        let child = if let Some(context) = error.attachment::<Context>() {
            context.clone()
        } else {
            let mut copy = Box::new(fpas_diagnostics::Diagnostic::error_without_source(
                error.code, "", None,
            ));
            state.ensure(&mut copy, None);
            copy.attachment::<Context>()
                .cloned()
                .unwrap_or_else(|| unreachable!("ensure attaches a cause"))
        };
        context.related = Some(Arc::new(Related {
            previous: context.related.take(),
            context: child,
            diagnostic,
            _lease: lease,
        }));
        context.related_count += 1;
    }
    primary.set_attachment(context);
}

impl Worker {
    /// Preserve the first available fault state; propagated errors retain their original cause.
    pub(in crate::vm) fn capture_failure(&self, mut error: VmError) -> VmError {
        if let Some(state) = self
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.failure_capture.get())
        {
            state.ensure(&mut error, Some(self));
        }
        error
    }

    /// Append an actual hosted FPAS caller without revisiting its cached callback frames.
    pub(in crate::vm) fn capture_failure_continuation(&self, error: &VmError) {
        let Some(context) = error.attachment::<Context>() else {
            return;
        };
        let Some(state) = context.owner.upgrade() else {
            return;
        };
        let mut draft = context
            .cause
            .draft
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Some(snapshot) = draft.as_mut()
            && snapshot.task_id == Some(self.task_id)
        {
            state.capture(self, snapshot);
        }
    }
}
