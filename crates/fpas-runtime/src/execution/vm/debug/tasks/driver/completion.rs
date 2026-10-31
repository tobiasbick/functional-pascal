//! Atomic program and task entry completion for the debugger task runtime.

use crate::{FunctionId, Value};

use super::{DebugTaskEvent, DebugTaskEventKind, DebugTaskRuntime, DebugTaskState};

impl DebugTaskRuntime {
    /// Complete an exactly matching runnable entry frame without dispatching work.
    ///
    /// Returns whether the root completed immediately. Pending group cleanup
    /// leaves the task inspectable and completes after debugger resume.
    /// A mismatch leaves the runtime and scheduler unchanged.
    pub(in crate::execution::vm::debug) fn complete_entry(
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
        if !root {
            slot.worker
                .require_transferable_values(std::slice::from_ref(&value))
                .ok()?;
        }
        let slot = self.tasks.get_mut(&task_id)?;
        let value = match slot.worker.finish_owned_groups(Ok(value)) {
            Ok(Some(value)) => value,
            Ok(None) => {
                slot.state = DebugTaskState::Waiting;
                slot.worker.supervision = None;
                return Some(false);
            }
            Err(error) => {
                slot.worker.hosted.cases.close_owner(task_id);
                slot.worker.pending_completion = Some(Err(error));
                slot.state = DebugTaskState::Runnable;
                slot.worker.supervision = None;
                return Some(false);
            }
        };
        slot.worker.hosted.cases.close_owner(task_id);
        slot.state = DebugTaskState::Completed;
        slot.worker.supervision = None;
        if retain_result && !root {
            self.scheduler.store_result(task_id, value.clone());
        }
        if root {
            self.root_result_roots = Some(
                slot.worker
                    .domain
                    .root_values(crate::RootKind::Native, vec![value.clone()]),
            );
            self.root_result = Some(value);
        } else if !slot.exited {
            slot.exited = true;
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Exited,
            });
        }
        slot.retire_execution();
        if root {
            self.cancel_after_root_completion();
        }
        Some(root)
    }

    pub(super) fn complete_failed_entry(&mut self, task_id: u64, value: Value) -> Option<bool> {
        let slot = self.tasks.get_mut(&task_id)?;
        if slot.exited || slot.state != DebugTaskState::Failed || slot.failure.is_none() {
            return None;
        }
        let root = task_id == 0;
        if !root {
            slot.worker
                .require_transferable_values(std::slice::from_ref(&value))
                .ok()?;
        }
        slot.worker.hosted.cases.close_owner(task_id);
        slot.failure = None;
        slot.worker.supervision = None;
        slot.state = DebugTaskState::Completed;
        if root {
            self.root_result_roots = Some(
                slot.worker
                    .domain
                    .root_values(crate::RootKind::Native, vec![value.clone()]),
            );
            self.root_result = Some(value);
        } else if !slot.exited {
            slot.exited = true;
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Exited,
            });
        }
        slot.retire_execution();
        if root {
            self.cancel_after_root_completion();
        }
        Some(root)
    }

    pub(super) fn cancel_after_root_completion(&mut self) {
        if self
            .tasks
            .values()
            .any(|slot| slot.worker.has_cleanup_obligations())
        {
            self.drain_disconnect_cleanup();
        }
        self.scheduler.finish_main();
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
            slot.retire_execution();
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
        self.retire_terminal_tickets();
        if let Some(worker) = self.worker(0) {
            worker.hosted.close_process_handles();
            worker.hosted.close_file_handles();
            worker.hosted.wait_process_handles();
        }
    }
}

#[cfg(test)]
mod tests;
