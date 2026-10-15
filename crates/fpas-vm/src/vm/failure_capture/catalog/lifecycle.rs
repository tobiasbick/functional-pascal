//! Task admission, handoff, completion and release without completed-task history.

use super::*;
use crate::vm::{VmError, failure_capture::CaptureState};
use std::sync::{Weak, atomic::Ordering};

impl CaptureState {
    /// Install the root and a non-running inspection template before starting worker threads.
    pub(in crate::vm) fn start_root(self: &Arc<Self>, worker: &mut Worker) {
        if let Some(scheduler) = &worker.scheduler {
            let _ = self.scheduler.set(Arc::downgrade(scheduler));
        }
        let mut template = worker.pool_template();
        template.scheduler = None;
        *self
            .catalog
            .template
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(template);
        worker.capture_record = self.admit(0, worker.function);
        worker.capture_root = true;
        if let Some(record) = &worker.capture_record {
            record.state.store(RUNNING, Ordering::Release);
        }
    }

    /// Reuse a current identity or admit bounded metadata for a new task.
    pub(super) fn admit(
        self: &Arc<Self>,
        id: u64,
        function: FunctionId,
    ) -> Option<Arc<TaskRecord>> {
        if self.finished.load(Ordering::Acquire) {
            return None;
        }
        let mut entries = self
            .catalog
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if self.finished.load(Ordering::Acquire) {
            return None;
        }
        if let Some(record) = entries.get(&id) {
            return Some(Arc::clone(record));
        }
        if entries.len() >= MAX_TASKS {
            if id == 0 {
                entries.pop_last();
            } else {
                return None;
            }
        }
        let bytes = std::mem::size_of::<TaskRecord>() + 80;
        let lease = match self.reserve_model(bytes) {
            Some(lease) => lease,
            None if id == 0 => self.reserve_terminal(bytes),
            None => return None,
        };
        let record = Arc::new(TaskRecord {
            id,
            function,
            state: AtomicU8::new(RUNNABLE),
            requested: AtomicBool::new(false),
            saved: Mutex::new(None),
            response: Mutex::new(None),
            failure: Mutex::new(Weak::new()),
            _lease: lease,
        });
        entries.insert(id, Arc::clone(&record));
        Some(record)
    }

    /// Mark a retained terminal outcome without retaining an additional diagnostic clone.
    pub(in crate::vm) fn task_finished(&self, id: u64, error: Option<&VmError>) {
        let record = self
            .catalog
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&id)
            .cloned();
        if let Some(record) = record {
            if let Some(error) = error {
                let cancelled = matches!(
                    error.code,
                    fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                        | fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
                );
                *record.failure.lock().unwrap_or_else(|p| p.into_inner()) = if cancelled {
                    Weak::new()
                } else {
                    error
                        .attachment::<super::super::identity::Context>()
                        .filter(|context| context.cause.freeze().task_id == Some(id))
                        .map_or_else(Weak::new, |context| Arc::downgrade(&context.cause))
                };
                record.state.store(
                    if cancelled { CANCELLED } else { FAILED },
                    Ordering::Release,
                );
            } else {
                record
                    .response
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .take();
                *record.failure.lock().unwrap_or_else(|p| p.into_inner()) = Weak::new();
                record.state.store(COMPLETED, Ordering::Release);
            }
        }
        self.catalog.notify_changed();
    }

    /// Forget an outcome when the existing scheduler owner releases it.
    pub(in crate::vm) fn release_task(&self, id: u64) {
        self.catalog
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&id);
    }

    /// Release inspection ownership after a successful or failed top-level run.
    pub(in crate::vm) fn finish_catalog(&self) {
        self.finished.store(true, Ordering::Release);
        self.catalog
            .template
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        self.catalog
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        self.root_globals
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
    }
}

impl Worker {
    /// Mark an aborted worker only after its execution owner is actually being discarded.
    pub(in crate::vm) fn capture_aborted_task(&self) {
        if let Some(record) = &self.capture_record {
            record
                .response
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .take();
            *record.failure.lock().unwrap_or_else(|p| p.into_inner()) = Weak::new();
            record.state.store(CANCELLED, Ordering::Release);
        }
    }

    /// Track exceptional pool disposal without reading a worker during unwinding.
    pub(in crate::vm) fn capture_pool_guard(&self) -> Option<PoolCaptureGuard> {
        Some(PoolCaptureGuard {
            record: Arc::clone(self.capture_record.as_ref()?),
            state: Arc::clone(self.scheduler.as_ref()?.failure_capture.get()?),
        })
    }

    /// Retain a distinct main-task propagation stack before its destructive scope cleanup.
    pub(in crate::vm) fn capture_root_propagation(&self, error: &VmError) {
        if !self.capture_root || self.capture_callback {
            return;
        }
        let Some(state) = self
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.failure_capture.get())
        else {
            return;
        };
        let original = error
            .attachment::<super::super::identity::Context>()
            .map(|context| context.cause.freeze());
        // Scope cleanup can replace a body cancellation with its child's original failure.
        if (original
            .as_ref()
            .is_some_and(|snapshot| snapshot.task_id != Some(self.task_id))
            || matches!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                    | fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN
            ))
            && let Some(record) = &self.capture_record
        {
            *record.response.lock().unwrap_or_else(|p| p.into_inner()) =
                Some(state.capture_handling(self));
        }
        state.capture_globals(self);
    }

    /// Keep root globals available through server-finalization errors after normal return.
    pub(in crate::vm) fn capture_root_outcome(
        &self,
        outcome: &Result<crate::vm::Execution, VmError>,
    ) {
        let Some(state) = self
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.failure_capture.get())
        else {
            return;
        };
        self.capture_root_globals(outcome.is_err());
        state.task_finished(0, outcome.as_ref().err());
    }

    /// Preserve globals for errors first selected during joins, without changing root outcome.
    pub(in crate::vm) fn capture_root_globals(&self, failed: bool) {
        let Some(state) = self
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.failure_capture.get())
        else {
            return;
        };
        if failed
            && state
                .root_globals
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_none()
        {
            state.capture_globals(self);
        }
    }

    /// Answer only an active collection request, on this task's own execution thread.
    pub(in crate::vm) fn capture_peer_boundary(&self) {
        let Some(record) = &self.capture_record else {
            return;
        };
        if !record.requested.swap(false, Ordering::AcqRel) {
            return;
        }
        let Some(state) = self
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.failure_capture.get())
        else {
            return;
        };
        let snapshot = state.capture_peer(self);
        let mut response = record.response.lock().unwrap_or_else(|p| p.into_inner());
        if state.catalog.collecting.load(Ordering::Acquire) {
            *response = Some(snapshot);
        }
        state.catalog.notify_changed();
    }
}

/// A pool panic ends the execution owner even when no VM diagnostic existed at its fault.
pub(in crate::vm) struct PoolCaptureGuard {
    record: Arc<TaskRecord>,
    state: Arc<CaptureState>,
}

impl Drop for PoolCaptureGuard {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.record
                .response
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .take();
            *self
                .record
                .failure
                .lock()
                .unwrap_or_else(|p| p.into_inner()) = Weak::new();
            self.record.state.store(FAILED, Ordering::Release);
            self.state.catalog.notify_changed();
        }
    }
}
