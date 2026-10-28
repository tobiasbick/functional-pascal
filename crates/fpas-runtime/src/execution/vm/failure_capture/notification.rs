//! Terminal selection is observed only after the runtime has selected its escaping error.

use super::{CaptureState, FailureObservation, identity::Context};
use crate::execution::vm::{Vm, VmError};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, atomic::Ordering},
};

impl CaptureState {
    /// Publish once, outside runtime locks; a host observer cannot replace the VM diagnostic.
    pub(in crate::execution::vm) fn notify(self: &Arc<Self>, error: &VmError) {
        if self.notified.swap(true, Ordering::AcqRel) {
            return;
        }
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let context = error.attachment::<Context>().cloned().unwrap_or_else(|| {
                let mut copy = Box::new(fpas_diagnostics::Diagnostic::error_without_source(
                    error.code, "", None,
                ));
                self.ensure(&mut copy, None);
                copy.attachment::<Context>()
                    .cloned()
                    .unwrap_or_else(|| unreachable!("ensure attaches a cause"))
            });
            let lease = self.reserve_terminal(std::mem::size_of::<FailureObservation>() + 16);
            let mut remaining = super::MAX_RELATED;
            let mut observation = observe(
                self,
                &context,
                self.terminal_presentation(error),
                lease,
                &mut remaining,
                0,
            );
            observation.task_catalog = Some(self.collect_tasks(&observation));
            (self.observer)(Arc::new(observation));
        }));
    }
}

fn observe(
    state: &Arc<CaptureState>,
    context: &Context,
    error: Arc<super::FailureDiagnostic>,
    lease: super::Lease,
    remaining: &mut usize,
    depth: usize,
) -> FailureObservation {
    let cause = context.cause.freeze();
    let maximum = if depth >= super::MAX_FRAMES {
        0
    } else {
        context.related_count.min(*remaining)
    };
    // Charge vector/model storage before creating either the traversal list or its output.
    let (count, collection_lease) = state.reserve_collection(
        maximum,
        std::mem::size_of::<FailureObservation>()
            + std::mem::size_of::<&super::identity::Related>(),
    );
    let mut entries = Vec::with_capacity(count);
    let mut next = context.related.as_deref();
    for _ in 0..context.related_count.saturating_sub(count) {
        next = next.and_then(|entry| entry.previous.as_deref());
    }
    for _ in 0..count {
        if let Some(entry) = next {
            entries.push(entry);
            next = entry.previous.as_deref();
        }
    }
    let mut related_errors = Vec::with_capacity(count);
    let mut omitted = context.related_omitted + context.related_count.saturating_sub(entries.len());
    for entry in entries.into_iter().rev() {
        if *remaining == 0 {
            omitted += 1;
            continue;
        }
        *remaining -= 1;
        let Some(child_lease) = state.reserve_model(std::mem::size_of::<FailureObservation>())
        else {
            omitted += 1;
            continue;
        };
        related_errors.push(observe(
            state,
            &entry.context,
            Arc::clone(&entry.diagnostic),
            child_lease,
            remaining,
            depth + 1,
        ));
    }
    FailureObservation {
        task_catalog: None,
        error,
        cause,
        related_errors,
        related_errors_omitted: omitted,
        _lease: lease,
        _collection_lease: collection_lease,
    }
}

impl Vm {
    /// Enable original-fault capture for [`Vm::run`] and one notification before shutdown joins.
    ///
    /// Returns `false` if execution has started or an observer is already installed. The
    /// observer receives immutable owned data and must return promptly. Handled failures
    /// and successful runs do not notify. Observer panics cannot change the VM outcome.
    pub fn set_failure_observer(
        &mut self,
        observer: impl Fn(Arc<FailureObservation>) + Send + Sync + 'static,
    ) -> bool {
        !self.has_run
            && self
                .scheduler
                .failure_capture
                .set(CaptureState::new(observer))
                .is_ok()
    }

    /// Notify only when capture is enabled and the escaping error has been selected.
    pub(in crate::execution::vm) fn notify_failure(&self, error: &VmError) {
        if let Some(state) = self.scheduler.failure_capture.get() {
            state.notify(error);
        }
    }
}
