//! Stopped-state cancellation of one non-root debug task.

use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED;

use super::super::super::types::DebugTaskState;
use super::{DebugTaskRuntime, TaskSlot};
use crate::vm::tasks::TaskSuspension;
use crate::vm::{VmError, runtime_error};

/// Why a debugger cancel cannot change a task's lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::vm::debug) enum TaskCancelError {
    /// The identity is unknown, completed, or already cancelled.
    Unknown,
    /// A failed task cannot be cancelled.
    Failed,
    /// The main task is torn down by disconnect, not `task.cancel`.
    Root,
}

impl DebugTaskRuntime {
    /// Cancel one live non-root task without dispatching bytecode.
    pub(in crate::vm::debug) fn cancel_task(
        &mut self,
        task_id: u64,
    ) -> Result<(), TaskCancelError> {
        if task_id == 0 {
            return Err(TaskCancelError::Root);
        }
        {
            let slot = self
                .tasks
                .get_mut(&task_id)
                .ok_or(TaskCancelError::Unknown)?;
            match slot.state {
                DebugTaskState::Completed | DebugTaskState::Cancelled => {
                    return Err(TaskCancelError::Unknown);
                }
                DebugTaskState::Failed => return Err(TaskCancelError::Failed),
                DebugTaskState::Runnable
                | DebugTaskState::Running
                | DebugTaskState::Waiting
                | DebugTaskState::Sleeping => {}
            }
            if matches!(
                slot.worker.task_suspension,
                Some(TaskSuspension::ScopeUnwind { .. } | TaskSuspension::OwnerCleanup { .. })
            ) {
                return Err(TaskCancelError::Unknown);
            }
            let error = cancellation_error(slot, task_id);
            slot.worker.observed_task_failure.set(None);
            slot.worker.supervision = None;
            if !slot.worker.task_scopes.is_empty() {
                // Joining runs after resume; the stopped command never dispatches child code.
                if slot.worker.begin_scope_unwind(error.clone()).is_ok() {
                    slot.state = DebugTaskState::Waiting;
                    slot.paused = false;
                    return Ok(());
                }
            }
            match slot.worker.finish_owned_groups(Err(error.clone())) {
                Ok(None) => {
                    slot.state = DebugTaskState::Waiting;
                    slot.paused = false;
                    return Ok(());
                }
                Err(error) => {
                    super::failure::record(
                        slot,
                        task_id,
                        &error,
                        &self.scheduler,
                        &mut self.events,
                    );
                }
                Ok(Some(_)) => unreachable!("cancellation retains an error outcome"),
            }
        }
        Ok(())
    }
}

fn cancellation_error(slot: &TaskSlot, task_id: u64) -> VmError {
    let location = slot
        .worker
        .executable
        .executable()
        .source_map
        .lookup(slot.worker.current_address)
        .map(|run| SourceLocation::new(run.line, run.column))
        .unwrap_or_else(|| SourceLocation::new(1, 1));
    runtime_error(
        RUNTIME_TASK_CANCELLED,
        format!("Task {task_id} was cancelled by the debugger"),
        "Continue to let waiters observe the cancellation, or disconnect the session.",
        location,
    )
}
