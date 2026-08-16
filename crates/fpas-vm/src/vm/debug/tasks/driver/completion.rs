//! Atomic program and task entry completion for the debugger task runtime.

use fpas_bytecode::{FunctionId, Value};

use super::{DebugTaskEvent, DebugTaskEventKind, DebugTaskRuntime, DebugTaskState};

impl DebugTaskRuntime {
    /// Complete an exactly matching runnable entry frame without dispatching work.
    ///
    /// Returns whether the completed entry was the root. A mismatch leaves the
    /// runtime and scheduler unchanged.
    pub(in crate::vm::debug) fn complete_entry(
        &mut self,
        task_id: u64,
        function: FunctionId,
        base: usize,
        call_stack_len: usize,
        value: Value,
    ) -> Option<bool> {
        let slot = self.tasks.get(&task_id)?;
        let identity = slot
            .worker
            .call_stack
            .first()
            .map_or((slot.worker.function, slot.worker.base), |frame| {
                (frame.function, frame.base)
            });
        if slot.state != DebugTaskState::Runnable
            || identity != (function, base)
            || slot.worker.call_stack.len() != call_stack_len
        {
            return None;
        }

        let retain_result = slot.worker.retain_result;
        let root = task_id == 0;
        let slot = self.tasks.get_mut(&task_id)?;
        slot.state = DebugTaskState::Completed;
        if retain_result {
            self.scheduler.store_result(task_id, value.clone());
        }
        if root {
            self.root_result = Some(value);
            self.scheduler.finish_main();
        } else if !slot.exited {
            slot.exited = true;
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Exited,
            });
        }
        if root {
            self.cancel_after_root_completion();
        }
        Some(root)
    }

    pub(super) fn complete_failed_entry(&mut self, task_id: u64, value: Value) -> Option<bool> {
        let slot = self.tasks.get_mut(&task_id)?;
        if slot.state != DebugTaskState::Failed || slot.failure.is_none() {
            return None;
        }
        slot.failure = None;
        slot.state = DebugTaskState::Completed;
        let root = task_id == 0;
        if root {
            self.root_result = Some(value);
            self.scheduler.finish_main();
        } else if !slot.exited {
            slot.exited = true;
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Exited,
            });
        }
        if root {
            self.cancel_after_root_completion();
        }
        Some(root)
    }

    pub(super) fn cancel_after_root_completion(&mut self) {
        for (&task_id, slot) in &mut self.tasks {
            if task_id == 0
                || matches!(
                    slot.state,
                    DebugTaskState::Completed | DebugTaskState::Failed | DebugTaskState::Cancelled
                )
            {
                continue;
            }
            slot.state = DebugTaskState::Cancelled;
            slot.worker.task_suspension = None;
            if slot.worker.retain_result {
                self.scheduler.cancel_result(task_id);
            }
            if !slot.exited {
                slot.exited = true;
                self.events.push(DebugTaskEvent {
                    task_id,
                    kind: DebugTaskEventKind::Exited,
                });
            }
        }
    }
}
