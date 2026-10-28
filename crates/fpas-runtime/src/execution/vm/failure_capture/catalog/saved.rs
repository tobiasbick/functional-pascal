//! Scheduler ownership wrappers retain saved state without copying registers.

use super::super::CaptureState;
use super::*;
use crate::execution::vm::tasks::TaskSuspension;
use std::sync::atomic::Ordering;

/// Uninstrumented tasks remain direct values; only enabled tasks use synchronized ownership.
#[expect(
    clippy::large_enum_variant,
    reason = "Keeping ordinary tasks inline avoids a heap allocation when failure capture is disabled"
)]
pub(in crate::execution::vm) enum ScheduledTask {
    /// Ordinary scheduler storage without D1 allocation.
    Direct(TaskState),
    /// Saved original task state, shared with the optional inspection catalog.
    Observed(Arc<TaskRecord>),
}

impl ScheduledTask {
    /// A plain completed sleep becomes runnable; poll waits and retry admission stay pending.
    pub(in crate::execution::vm) fn timer_ready(&self) {
        if let Self::Observed(record) = self {
            let saved = record.saved.lock().unwrap_or_else(|p| p.into_inner());
            if saved.as_ref().is_some_and(|task| {
                task.suspension.is_none()
                    && !task
                        .supervision
                        .as_ref()
                        .is_some_and(|state| state.is_admission_pending())
            }) {
                record.state.store(RUNNABLE, Ordering::Release);
            }
        }
    }

    /// Transfer the saved state to a worker, retaining its stable catalog identity.
    pub(in crate::execution::vm) fn take(self) -> TaskState {
        match self {
            Self::Direct(task) => task,
            Self::Observed(record) => {
                let mut saved = record.saved.lock().unwrap_or_else(|p| p.into_inner());
                let mut task = saved
                    .take()
                    .unwrap_or_else(|| unreachable!("a scheduled task has one owner"));
                record.state.store(RUNNING, Ordering::Release);
                task.capture_record = Some(Arc::clone(&record));
                task
            }
        }
    }
}

impl CaptureState {
    /// Save the original registers under their owner lock, without taking a value snapshot.
    pub(in crate::execution::vm) fn save(
        self: &Arc<Self>,
        mut task: TaskState,
        timer: bool,
    ) -> ScheduledTask {
        let _maintenance = crate::defer_maintenance();
        if self.finished.load(Ordering::Acquire) {
            task.capture_record = None;
            return ScheduledTask::Direct(task);
        }
        let record = task
            .capture_record
            .take()
            .or_else(|| self.admit(task.id, task.function));
        let Some(record) = record else {
            return ScheduledTask::Direct(task);
        };
        let state = match task.suspension.as_ref() {
            Some(TaskSuspension::Yield) => RUNNABLE,
            Some(TaskSuspension::Sleep { .. } | TaskSuspension::SupervisionBackoff { .. }) => {
                SLEEPING
            }
            Some(_) => WAITING,
            None if timer => SLEEPING,
            None => RUNNABLE,
        };
        {
            let mut saved = record.saved.lock().unwrap_or_else(|p| p.into_inner());
            *saved = Some(task);
            record.state.store(state, Ordering::Release);
        }
        ScheduledTask::Observed(record)
    }
}

impl crate::TraceRoots for ScheduledTask {
    fn trace_roots(
        &self,
        visit: &mut dyn FnMut(&crate::Value),
    ) -> Result<(), crate::OwnershipError> {
        match self {
            Self::Direct(task) => task.trace_roots(visit),
            Self::Observed(record) => {
                let saved = record.saved.try_lock().map_err(|error| match error {
                    std::sync::TryLockError::WouldBlock => crate::OwnershipError::Busy,
                    std::sync::TryLockError::Poisoned(_) => crate::OwnershipError::Poisoned,
                })?;
                if let Some(task) = saved.as_ref() {
                    task.trace_roots(visit)?;
                }
                Ok(())
            }
        }
    }
}
