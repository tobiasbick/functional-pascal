//! Stopped-state per-task pause, resume, and cancellation without admission.

use super::*;
use crate::vm::debug::tasks::{TaskCancelError, TaskHoldError};

impl DebugSession {
    /// Hold `task_id` so later session-wide continue and peer steps skip it.
    ///
    /// Unknown, completed, cancelled, and failed identities reject without
    /// mutating workers or the stop generation.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-task command error.
    pub fn pause_task(&mut self, task_id: u64) -> Result<(), DebugSessionError> {
        self.require_stopped("task.pause")?;
        self.runtime
            .pause_task(task_id)
            .map_err(|error| hold_error("task.pause", task_id, error))
    }

    /// Clear the hold on `task_id` so later continue and steps may dispatch it.
    ///
    /// This does not resume the session. Unknown, completed, cancelled, and
    /// failed identities reject without mutating workers or the stop generation.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-task command error.
    pub fn resume_task(&mut self, task_id: u64) -> Result<(), DebugSessionError> {
        self.require_stopped("task.resume")?;
        self.runtime
            .resume_task(task_id)
            .map_err(|error| hold_error("task.resume", task_id, error))
    }

    /// Cancel one live non-root task at the current stop.
    ///
    /// The command marks the task cancelled, publishes a waiter-visible failure
    /// when the task retains a result, and emits an exit event. It does not
    /// dispatch bytecode, drain spawns, or wake waiters. Unknown, completed,
    /// cancelled, failed, and root identities reject without mutation.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-task command error.
    pub fn cancel_task(&mut self, task_id: u64) -> Result<(), DebugSessionError> {
        self.require_stopped("task.cancel")?;
        self.runtime
            .cancel_task(task_id)
            .map_err(|error| cancel_error(task_id, error))?;
        self.drop_cancelled_inspection(task_id);
        Ok(())
    }

    fn drop_cancelled_inspection(&mut self, task_id: u64) {
        self.inspections.remove(&task_id);
        if self.inspection_task_id == task_id {
            self.inspection_task_id = self.inspections.keys().next().copied().unwrap_or(0);
        }
    }
}

fn hold_error(command: &'static str, task_id: u64, error: TaskHoldError) -> DebugSessionError {
    match error {
        TaskHoldError::Unknown => unknown_task(task_id),
        TaskHoldError::Failed => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: format!("debug command `{command}` cannot target failed task {task_id}"),
            hint: "Select a live inspectable task, or launch a new debug session.".to_string(),
        },
    }
}

fn cancel_error(task_id: u64, error: TaskCancelError) -> DebugSessionError {
    match error {
        TaskCancelError::Unknown => unknown_task(task_id),
        TaskCancelError::Failed => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: format!("debug command `task.cancel` cannot target failed task {task_id}"),
            hint: "Select a live inspectable task, or disconnect the session.".to_string(),
        },
        TaskCancelError::Root => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: "debug command `task.cancel` cannot target the main task".to_string(),
            hint: "Disconnect the debug session to cancel the root task.".to_string(),
        },
    }
}
