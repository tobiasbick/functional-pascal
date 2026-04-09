//! `Std.Task.Wait` / `Std.Task.WaitAll` execution.
//!
//! **Documentation:** `docs/pascal/std/task.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::{TaskResultPoll, Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN,
};
use std::time::Duration;

/// When yield does not reschedule, block briefly on the condvar instead of hot-spinning.
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(1);

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
            TaskResultPoll::Pending if self.shared.is_shutdown() => {
                return Err(runtime_error(
                    RUNTIME_VM_SHUTDOWN,
                    "Execution aborted: the waited task failed",
                    "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                    line,
                ));
            }
            TaskResultPoll::Pending => {
                self.push(Value::Task(task_id))?;
                self.ip -= 1;
                if !self.exec_yield() {
                    self.shared.wait_for_task_progress(WAIT_POLL_INTERVAL);
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

        let all_done = task_ids
            .iter()
            .all(|task_id| self.shared.task_completion_recorded(*task_id));

        if all_done {
            // `WaitAll` observes completion but does not consume task results.
        } else if self.shared.is_shutdown() {
            return Err(runtime_error(
                RUNTIME_VM_SHUTDOWN,
                "Execution aborted: a waited task failed",
                "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                line,
            ));
        } else {
            self.push(Value::Array(tasks))?;
            self.ip -= 1;
            if !self.exec_yield() {
                self.shared.wait_for_task_progress(WAIT_POLL_INTERVAL);
            }
        }
        Ok(())
    }
}

pub(super) fn pop_task_id(vm: &mut Worker, line: SourceLocation) -> Result<u64, VmError> {
    let value = vm.pop(line)?;
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
