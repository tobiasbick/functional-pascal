use super::super::super::super::{Vm, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Vm {
    pub(in super::super) fn exec_task_wait(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let task_id = self.pop_task_id(line)?;

        if let Some(result) = self.task_results.remove(&task_id) {
            self.push(result)?;
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
                self.task_results.contains_key(id)
            } else {
                true
            }
        });

        if all_done {
            let results: Vec<Value> = tasks
                .iter()
                .map(|value| {
                    if let Value::Task(id) = value {
                        self.task_results.remove(id).unwrap_or(Value::Unit)
                    } else {
                        Value::Unit
                    }
                })
                .collect();
            self.push(Value::Array(results))?;
        } else {
            self.push(Value::Array(tasks))?;
            self.ip -= 1;
            self.exec_yield();
        }
        Ok(())
    }
}

pub(super) fn pop_task_id(vm: &mut Vm, line: SourceLocation) -> Result<u64, VmError> {
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
