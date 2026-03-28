use super::super::super::super::{TaskContext, Vm, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

impl Vm {
    pub(in super::super) fn exec_spawn_task(
        &mut self,
        argc: u8,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let (name, captures) = match func {
            Value::Function { name, captures } => (name, captures),
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Expected function value for `go`, got `{}`",
                        other.type_name()
                    ),
                    "Use `go FunctionName(args)` to spawn a task.",
                    line,
                ));
            }
        };

        let (code_start, _) = self.chunk.functions.get(&name).copied().ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_TASK,
                format!("Function `{name}` not found for task spawn"),
                "Ensure the function is defined before spawning it as a task.",
                line,
            )
        })?;

        let arg_count = argc as usize;
        let args_start = self.stack.len().saturating_sub(arg_count);
        let args: Vec<Value> = self.stack.drain(args_start..).collect();
        let mut task_stack = Vec::with_capacity(arg_count + captures.len());
        task_stack.extend(args);
        task_stack.extend(captures);

        let task_id = self.next_task_id;
        self.next_task_id += 1;

        self.tasks.push(TaskContext {
            id: task_id,
            ip: code_start,
            stack: task_stack,
            call_stack: Vec::new(),
        });

        self.push(Value::Task(task_id))?;
        Ok(())
    }
}
