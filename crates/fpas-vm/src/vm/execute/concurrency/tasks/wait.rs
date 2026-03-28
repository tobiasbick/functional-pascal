use super::super::super::super::diagnostics::VmError;
use super::super::super::super::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN};

impl Worker {
    pub(in super::super) fn exec_task_wait(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let task_id = self.pop_task_id(line)?;

        if let Some(result) = self.shared.take_task_result(task_id) {
            self.push(result)?;
        } else if self.shared.is_shutdown() {
            return Err(runtime_error(
                RUNTIME_VM_SHUTDOWN,
                "Execution aborted: the waited task failed",
                "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                line,
            ));
        } else {
            self.push(Value::Task(task_id))?;
            self.ip -= 1;
            self.exec_yield();
        }
        Ok(())
    }

    pub(in super::super) fn exec_task_wait_all(
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

        let all_done = tasks.iter().all(|value| {
            if let Value::Task(id) = value {
                self.shared.has_task_result(*id)
            } else {
                true
            }
        });

        if all_done {
            let results: Vec<Value> = tasks
                .iter()
                .map(|value| {
                    if let Value::Task(id) = value {
                        self.shared.take_task_result(*id).unwrap_or(Value::Unit)
                    } else {
                        Value::Unit
                    }
                })
                .collect();
            self.push(Value::Array(results))?;
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
            self.exec_yield();
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
