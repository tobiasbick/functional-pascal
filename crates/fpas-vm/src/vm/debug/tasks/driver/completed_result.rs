//! Stable retained-result identity for completed debugger tasks.

use fpas_bytecode::{FunctionId, Value};

use super::{DebugTaskRuntime, DebugTaskState};
use crate::vm::tasks::RetainedResultReplacement;

/// Eligibility failure for a requested completed task result.
pub(in crate::vm::debug) enum CompletedResultTargetError {
    /// No debugger task uses the supplied identity.
    UnknownTask,
    /// The task has not completed successfully.
    NotCompleted,
    /// The task was detached and owns no retained result.
    NotRetained,
}

impl DebugTaskRuntime {
    /// Return the exact entry function that declares a replaceable task result.
    pub(in crate::vm::debug) fn completed_result_function(
        &self,
        task_id: u64,
    ) -> Result<FunctionId, CompletedResultTargetError> {
        let slot = self
            .tasks
            .get(&task_id)
            .ok_or(CompletedResultTargetError::UnknownTask)?;
        if slot.state != DebugTaskState::Completed {
            return Err(CompletedResultTargetError::NotCompleted);
        }
        if !slot.worker.retain_result {
            return Err(CompletedResultTargetError::NotRetained);
        }
        Ok(slot.entry_function)
    }

    /// Replace one unconsumed retained result after session-level type validation.
    pub(in crate::vm::debug) fn replace_completed_result(
        &self,
        task_id: u64,
        value: Value,
    ) -> RetainedResultReplacement {
        self.scheduler.replace_available_result(task_id, value)
    }
}
