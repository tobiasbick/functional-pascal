//! `Std.Task.Wait` / `Std.Task.WaitAll` execution.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::execute::StepResult;
use crate::vm::{TaskResultPoll, Worker, internal_error, runtime_error};
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
            TaskResultPoll::Consumed => {
                return Err(runtime_error(
                    RUNTIME_INVALID_TASK,
                    format!("Task {task_id} was already awaited"),
                    "Wait on each task handle only once, or keep the result in a variable after waiting.",
                    line,
                ));
            }
            TaskResultPoll::Unknown if self.shared.is_shutdown() => {
                return Err(waited_task_failed(line));
            }
            TaskResultPoll::Unknown => {
                return Err(unknown_task_error(task_id, line));
            }
            TaskResultPoll::Pending if self.shared.is_shutdown() => {
                return Err(waited_task_failed(line));
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
                        TaskResultPoll::Consumed => {
                            return Err(runtime_error(
                                RUNTIME_INVALID_TASK,
                                format!("Task {task_id} was already awaited"),
                                "Wait on each task handle only once, or keep the result in a variable after waiting.",
                                line,
                            ));
                        }
                        TaskResultPoll::Unknown if self.shared.is_shutdown() => {
                            return Err(waited_task_failed(line));
                        }
                        TaskResultPoll::Unknown => {
                            return Err(unknown_task_error(task_id, line));
                        }
                        TaskResultPoll::Pending if self.shared.is_shutdown() => {
                            return Err(waited_task_failed(line));
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

        if let Some(task_id) = self.shared.first_unknown_task(&task_ids) {
            if self.shared.is_shutdown() {
                return Err(waited_task_failed(line));
            }
            return Err(unknown_task_error(task_id, line));
        }

        let all_done = self.shared.all_tasks_recorded(&task_ids);

        if all_done {
            // `WaitAll` observes completion but does not consume task results.
        } else if self.shared.is_shutdown() {
            return Err(waited_task_failed(line));
        } else {
            self.push(Value::Array(tasks))?;
            self.ip -= 1;
            loop {
                if self.sync_call_depth == 0 && self.exec_yield() {
                    return Ok(());
                }
                if self.sync_call_depth > 0 {
                    while self.help_run_one_ready_task(line)? {}
                }
                if self.shared.all_tasks_recorded(&task_ids) {
                    let _ = self.pop(line)?;
                    self.ip += 1;
                    return Ok(());
                }
                if self.shared.is_shutdown() {
                    return Err(waited_task_failed(line));
                }
                if self.sync_call_depth > 0 {
                    self.shared.wait_until_all_tasks_recorded_strict(&task_ids);
                } else {
                    self.shared.wait_until_all_tasks_recorded(&task_ids);
                }
            }
        }
        Ok(())
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

            if self.shared.is_shutdown() {
                return Ok(());
            }
            let code_len = self.shared.chunk.code().len();
            if self.ip >= code_len {
                let result = self.stack.pop().unwrap_or(Value::Unit);
                if self.current_task_retain_result {
                    self.shared.store_task_result(helped_id, result);
                }
                return Ok(());
            }

            match self.exec_one(caller_line)? {
                StepResult::Continue => {}
                StepResult::Halt => {
                    return Err(internal_error(
                        "Halt while helping a ready task during Wait",
                        "Spawned tasks must return with `Return`, not `Halt`.",
                        caller_line,
                    ));
                }
                StepResult::Suspended => {
                    self.task_suspended = false;
                    return Ok(());
                }
                StepResult::Return => {
                    let location = self.current_location;
                    let return_val = self.pop(location)?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_slot);
                        self.push(return_val)?;
                        self.ip = frame.return_ip;
                    } else {
                        if self.current_task_retain_result {
                            self.shared.store_task_result(helped_id, return_val);
                        }
                        return Ok(());
                    }
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
