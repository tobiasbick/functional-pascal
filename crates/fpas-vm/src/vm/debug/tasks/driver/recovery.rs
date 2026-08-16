//! Exact failed-task transitions used by explicit debugger recovery.

use fpas_bytecode::Value;

use super::{DebugTaskRuntime, DebugTaskState};
use crate::vm::VmError;
use crate::vm::debug::forced_return::{PreparedEntryCompletion, PreparedSelection, apply_prepared};

impl DebugTaskRuntime {
    /// Replace an exact failed callee with a prepared return and make the task runnable.
    pub(in crate::vm::debug) fn recover_failed_return(
        &mut self,
        task_id: u64,
        expected: &VmError,
        prepared: &PreparedSelection,
        value: Value,
    ) -> bool {
        let Some(slot) = self.tasks.get(&task_id) else {
            return false;
        };
        if slot.state != DebugTaskState::Failed
            || slot.failure.as_ref() != Some(expected)
            || crate::vm::debug::forced_return::prepare_selection(&slot.worker, prepared.depth)
                .as_ref()
                != Ok(prepared)
        {
            return false;
        }
        if slot.worker.retain_result && !self.scheduler.recover_failure(task_id, expected) {
            return false;
        }
        let Some(slot) = self.tasks.get_mut(&task_id) else {
            unreachable!("validated failed task remains retained")
        };
        apply_prepared(&mut slot.worker, prepared, value);
        slot.state = DebugTaskState::Runnable;
        slot.failure = None;
        true
    }

    /// Complete an exact failed program or task entry with a replacement result.
    pub(in crate::vm::debug) fn recover_failed_entry(
        &mut self,
        expected: &VmError,
        prepared: PreparedEntryCompletion,
        value: Value,
    ) -> Option<bool> {
        let slot = self.tasks.get(&prepared.task_id)?;
        let identity = slot
            .worker
            .call_stack
            .first()
            .map_or((slot.worker.function, slot.worker.base), |frame| {
                (frame.function, frame.base)
            });
        if slot.state != DebugTaskState::Failed
            || slot.failure.as_ref() != Some(expected)
            || identity != (prepared.function, prepared.base)
            || slot.worker.call_stack.len() != prepared.call_stack_len
        {
            return None;
        }
        if slot.worker.retain_result
            && !self
                .scheduler
                .replace_failure(prepared.task_id, expected, value.clone())
        {
            return None;
        }
        self.complete_failed_entry(prepared.task_id, value)
    }
}
