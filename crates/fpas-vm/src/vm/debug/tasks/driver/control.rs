//! Per-task hold flags that persist across session-wide continue until resume.

use super::super::super::types::DebugTaskState;
use super::DebugTaskRuntime;

/// Why a per-task pause or resume cannot change the hold flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::vm::debug) enum TaskHoldError {
    /// The identity is unknown, completed, or cancelled.
    Unknown,
    /// A failed task cannot be paused or resumed.
    Failed,
}

impl DebugTaskRuntime {
    /// Hold `task_id` so later continue and peer steps skip it.
    pub(in crate::vm::debug) fn pause_task(&mut self, task_id: u64) -> Result<(), TaskHoldError> {
        self.set_paused(task_id, true)
    }

    /// Clear the hold on `task_id` so later continue and steps may dispatch it.
    pub(in crate::vm::debug) fn resume_task(&mut self, task_id: u64) -> Result<(), TaskHoldError> {
        self.set_paused(task_id, false)
    }

    /// Whether `task_id` is currently skipped by the scheduler.
    pub(in crate::vm::debug) fn task_is_paused(&self, task_id: u64) -> bool {
        self.tasks.get(&task_id).is_some_and(|slot| slot.paused)
    }

    fn set_paused(&mut self, task_id: u64, paused: bool) -> Result<(), TaskHoldError> {
        let slot = self.tasks.get_mut(&task_id).ok_or(TaskHoldError::Unknown)?;
        match slot.state {
            DebugTaskState::Completed | DebugTaskState::Cancelled => {
                return Err(TaskHoldError::Unknown);
            }
            DebugTaskState::Failed => return Err(TaskHoldError::Failed),
            DebugTaskState::Runnable
            | DebugTaskState::Running
            | DebugTaskState::Waiting
            | DebugTaskState::Sleeping => {}
        }
        slot.paused = paused;
        Ok(())
    }
}
