//! `Std.Task.Wait` / `Std.Task.WaitAll` execution.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::execute::transition::{ExecutionContext, ExecutionTransition};
use crate::vm::{TaskBatchPoll, TaskResultPoll, Worker, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN,
};

impl Worker {
    pub(in crate::vm::execute::concurrency) fn exec_task_wait(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let task_id = self.pop_task_id(line)?;

        match self.shared.poll_task_result(task_id) {
            TaskResultPoll::Available(result) => {
                self.push(result)?;
            }
            TaskResultPoll::Failed(error) => {
                return Err(error);
            }
            TaskResultPoll::Consumed => {
                return Err(runtime_error(
                    RUNTIME_INVALID_TASK,
                    format!("Task {task_id} was already awaited"),
                    "Wait on each task handle only once, or keep the result in a variable after waiting.",
                    line,
                ));
            }
            TaskResultPoll::Unknown if self.shared.is_shutdown() => {
                return Err(self.task_failure_or_shutdown(task_id, line));
            }
            TaskResultPoll::Unknown => {
                return Err(unknown_task_error(task_id, line));
            }
            TaskResultPoll::Pending if self.shared.is_shutdown() => {
                return Err(self.task_failure_or_shutdown(task_id, line));
            }
            TaskResultPoll::Pending => {
                self.push(Value::Task(task_id))?;
                self.ip -= 1;
                loop {
                    if self.sync_call_depth == 0 && self.exec_yield() {
                        return Ok(());
                    }
                    // Under a sync callback we cannot yield. Run queued work in-place
                    // so pool_size == 1 cannot livelock, then park without spinning.
                    if self.sync_call_depth > 0 {
                        while self.help_run_one_ready_task(line)? {}
                    }
                    match self.shared.poll_task_result(task_id) {
                        TaskResultPoll::Available(result) => {
                            let _ = self.pop(line)?;
                            self.push(result)?;
                            self.ip += 1;
                            return Ok(());
                        }
                        TaskResultPoll::Failed(error) => {
                            return Err(error);
                        }
                        TaskResultPoll::Consumed => {
                            return Err(runtime_error(
                                RUNTIME_INVALID_TASK,
                                format!("Task {task_id} was already awaited"),
                                "Wait on each task handle only once, or keep the result in a variable after waiting.",
                                line,
                            ));
                        }
                        TaskResultPoll::Unknown if self.shared.is_shutdown() => {
                            return Err(self.task_failure_or_shutdown(task_id, line));
                        }
                        TaskResultPoll::Unknown => {
                            return Err(unknown_task_error(task_id, line));
                        }
                        TaskResultPoll::Pending if self.shared.is_shutdown() => {
                            return Err(self.task_failure_or_shutdown(task_id, line));
                        }
                        TaskResultPoll::Pending if self.sync_call_depth > 0 => {
                            self.shared.wait_until_task_result_ready_strict(task_id);
                        }
                        TaskResultPoll::Pending => {
                            self.shared.wait_until_task_result_ready(task_id);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub(in crate::vm::execute::concurrency) fn exec_task_wait_all(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let arr = self.pop(line)?;
        let Value::Array(tasks) = arr else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected array for WaitAll, got `{}`", arr.type_name()),
                "Pass an array of task handles to `Std.Task.WaitAll`.",
                line,
            ));
        };

        let mut task_ids = Vec::with_capacity(tasks.len());
        for value in &tasks {
            let Value::Task(id) = value else {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Expected every `Std.Task.WaitAll` element to be a task, got `{}`",
                        value.type_name()
                    ),
                    "Pass an array of task handles such as `[T1, T2, T3]`.",
                    line,
                ));
            };
            task_ids.push(*id);
        }
        task_ids.sort_unstable();
        task_ids.dedup();

        match self.shared.poll_task_batch(&task_ids) {
            TaskBatchPoll::Complete => return Ok(()),
            TaskBatchPoll::Failed(error) => return Err(error),
            TaskBatchPoll::Unknown(_) if self.shared.is_shutdown() => {
                return Err(self.task_batch_failure_or_shutdown(&task_ids, line));
            }
            TaskBatchPoll::Unknown(task_id) => return Err(unknown_task_error(task_id, line)),
            TaskBatchPoll::Pending if self.shared.is_shutdown() => {
                return Err(self.task_batch_failure_or_shutdown(&task_ids, line));
            }
            TaskBatchPoll::Pending => {}
        }

        self.push(Value::Array(tasks))?;
        self.ip -= 1;
        loop {
            if self.sync_call_depth == 0 && self.exec_yield() {
                return Ok(());
            }
            if self.sync_call_depth > 0 {
                while self.help_run_one_ready_task(line)? {}
            }
            match self.shared.poll_task_batch(&task_ids) {
                TaskBatchPoll::Complete => {
                    let _ = self.pop(line)?;
                    self.ip += 1;
                    return Ok(());
                }
                TaskBatchPoll::Failed(error) => return Err(error),
                TaskBatchPoll::Unknown(_) if self.shared.is_shutdown() => {
                    return Err(self.task_batch_failure_or_shutdown(&task_ids, line));
                }
                TaskBatchPoll::Unknown(task_id) => {
                    return Err(unknown_task_error(task_id, line));
                }
                TaskBatchPoll::Pending if self.shared.is_shutdown() => {
                    return Err(self.task_batch_failure_or_shutdown(&task_ids, line));
                }
                TaskBatchPoll::Pending => {}
            }
            if self.sync_call_depth > 0 {
                self.shared.wait_until_all_tasks_recorded_strict(&task_ids);
            } else {
                self.shared.wait_until_all_tasks_recorded(&task_ids);
            }
        }
    }

    fn pop_task_id(&mut self, line: SourceLocation) -> Result<u64, VmError> {
        let value = self.pop(line)?;
        match value {
            Value::Task(id) => Ok(id),
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected task, got `{}`", other.type_name()),
                "Pass a task handle from `go FunctionName(args)`.",
                line,
            )),
        }
    }

    fn task_failure_or_shutdown(&self, task_id: u64, line: SourceLocation) -> VmError {
        self.shared
            .task_failure(task_id)
            .unwrap_or_else(|| waited_task_failed(line))
    }

    fn task_batch_failure_or_shutdown(&self, task_ids: &[u64], line: SourceLocation) -> VmError {
        match self.shared.poll_task_batch(task_ids) {
            TaskBatchPoll::Failed(error) => error,
            _ => waited_task_failed(line),
        }
    }

    /// Run one ready task to completion (or suspension) without abandoning a sync callback.
    ///
    /// Used when `Wait`/`WaitAll` cannot `exec_yield` because `sync_call_depth > 0`.
    /// Returns `true` when a ready task was dequeued and driven.
    fn help_run_one_ready_task(&mut self, line: SourceLocation) -> Result<bool, VmError> {
        let Some(ready) = self.shared.try_dequeue_task() else {
            return Ok(false);
        };
        let saved = self.save_task();
        let saved_sync = self.sync_call_depth;
        let saved_pending = self.pending_entry_ip.take();
        self.load_task(ready);
        self.sync_call_depth = 0;

        let helped = self.run_helped_task_until_parked_or_done(line);
        if let Err(error) = &helped
            && self.current_task_retain_result
        {
            self.shared
                .store_task_failure(self.current_task_id, error.clone());
        }
        if helped.is_err() {
            self.shared.signal_runtime_failure();
        }
        self.sync_call_depth = saved_sync;
        self.pending_entry_ip = saved_pending;
        self.load_task(saved);
        helped?;
        Ok(true)
    }

    /// Drive the currently loaded helped task until it completes or cooperatively suspends.
    fn run_helped_task_until_parked_or_done(
        &mut self,
        caller_line: SourceLocation,
    ) -> Result<(), VmError> {
        let helped_id = self.current_task_id;
        loop {
            // Keep this helped task on the current worker; timeslice switching would
            // abandon the waiting sync callback's saved state.
            self.instructions_until_yield = u32::MAX;

            match self.advance_execution(ExecutionContext::SpawnedTask, caller_line)? {
                ExecutionTransition::Continue => {}
                ExecutionTransition::Cancelled => {
                    if self.current_task_retain_result {
                        self.shared.cancel_retained_task(helped_id);
                    }
                    return Ok(());
                }
                ExecutionTransition::Suspended => {
                    self.task_suspended = false;
                    return Ok(());
                }
                ExecutionTransition::Completed(return_value) => {
                    if self.current_task_retain_result {
                        self.shared.store_task_result(helped_id, return_value);
                    }
                    return Ok(());
                }
            }

            // Helped tasks must not steal other work via timeslice; keep them on this worker
            // until they finish or suspend so the waiting sync callback can resume.
            if self.current_task_id != helped_id {
                return Err(internal_error(
                    "Helped task switched identity during Wait",
                    "This indicates a VM scheduling bug. Please report it.",
                    caller_line,
                ));
            }
        }
    }
}

fn unknown_task_error(task_id: u64, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INVALID_TASK,
        format!("Task {task_id} was not created by this VM or does not retain a result"),
        "Pass a task handle returned by a `go` expression to `Wait` or `WaitAll`.",
        line,
    )
}

fn waited_task_failed(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_VM_SHUTDOWN,
        "Execution aborted: a waited task failed",
        "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
        line,
    )
}
