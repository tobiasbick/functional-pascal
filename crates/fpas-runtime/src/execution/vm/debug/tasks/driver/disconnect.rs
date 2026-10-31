//! Finish registered resources before discarding a disconnected debug session.

use super::{DebugSchedule, DebugTaskRuntime, DebugTaskState};
use crate::SourceLocation;
use crate::execution::vm::runtime_error;
use fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED;
use std::time::Duration;

impl DebugTaskRuntime {
    pub(super) fn drain_disconnect_cleanup(&mut self) {
        let ids: Vec<_> = self.tasks.keys().copied().collect();
        for id in ids {
            let Some(slot) = self.tasks.get_mut(&id) else {
                continue;
            };
            if matches!(
                slot.state,
                DebugTaskState::Completed | DebugTaskState::Cancelled | DebugTaskState::Failed
            ) {
                continue;
            }
            slot.paused = false;
            slot.worker.supervision = None;
            let error = runtime_error(
                RUNTIME_TASK_CANCELLED,
                format!("Task {id} was cancelled by debugger disconnect"),
                "Continue resource cleanup before terminating the session.",
                SourceLocation::new(1, 1),
            );
            if slot.worker.defer_cleanup_cancel(error.clone()) {
                continue;
            }
            let error = if let Some(suspension) = slot.worker.task_suspension.take() {
                match slot.worker.retire_cancelled_suspension(suspension, error) {
                    Ok(_) => {
                        slot.state = DebugTaskState::Waiting;
                        continue;
                    }
                    Err(error) => error,
                }
            } else {
                error
            };
            match slot.worker.begin_cleanup_failure(error) {
                Ok(()) => slot.state = DebugTaskState::Runnable,
                Err(error) => match slot.worker.begin_scope_unwind(error) {
                    Ok(()) => slot.state = DebugTaskState::Waiting,
                    Err(_) => {
                        slot.state = DebugTaskState::Cancelled;
                        if id != 0 && slot.worker.retain_result {
                            self.scheduler.cancel_result(id);
                        }
                        slot.retire_execution();
                    }
                },
            }
        }
        loop {
            if self.tasks.values().all(|slot| {
                matches!(
                    slot.state,
                    DebugTaskState::Completed | DebugTaskState::Cancelled | DebugTaskState::Failed
                )
            }) {
                break;
            }
            match self.schedule(None) {
                Ok(DebugSchedule::Runnable { task_id, .. }) => {
                    if let Err((id, _)) = self.dispatch(task_id) {
                        self.finish_disconnect_failure(id);
                    }
                }
                Err((id, _)) => self.finish_disconnect_failure(id),
                Ok(DebugSchedule::Idle(wait)) => self.wait(wait),
                Ok(
                    DebugSchedule::NoUnpausedWork
                    | DebugSchedule::Deadlock
                    | DebugSchedule::ClockExhausted,
                ) => self.wait(Duration::from_millis(1)),
            }
        }
    }

    fn finish_disconnect_failure(&mut self, id: u64) {
        if let Some(slot) = self.tasks.get_mut(&id) {
            slot.state = DebugTaskState::Cancelled;
            if id != 0 && slot.worker.retain_result {
                self.scheduler.cancel_result(id);
            }
            slot.retire_execution();
        }
    }
}
