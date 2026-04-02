use super::super::super::super::diagnostics::VmError;
use super::super::super::super::{
    TaskState, Worker, canonical_name, internal_error, runtime_error,
};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

impl Worker {
    /// Spawn a new task: pops function value + args, pushes `Value::Task(id)`.
    ///
    /// The task is placed on the shared queue for any worker thread to pick up.
    pub(in super::super) fn exec_spawn_task(
        &mut self,
        argc: u8,
        retain_result: bool,
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

        let (code_start, expected_arity) = self
            .shared
            .chunk
            .functions
            .get(name.as_str())
            .or_else(|| self.shared.chunk.functions.get(&canonical_name(&name)))
            .copied()
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_INVALID_TASK,
                    format!("Function `{name}` not found for task spawn"),
                    "Ensure the function is defined before spawning it as a task.",
                    line,
                )
            })?;

        if argc != expected_arity {
            return Err(runtime_error(
                RUNTIME_WRONG_CALL_ARITY,
                format!("Function `{name}` expects {expected_arity} arguments, got {argc}"),
                "Spawn the task with the declared number of arguments.",
                line,
            ));
        }

        let arg_count = argc as usize;
        let args = self.drain_stack_tail(arg_count, line).map_err(|_| {
            internal_error(
                format!("Task spawn for `{name}` expected {arg_count} argument(s) on the stack"),
                "This indicates invalid bytecode or a VM stack-layout bug. Please report it.",
                line,
            )
        })?;
        let mut task_stack = Vec::with_capacity(arg_count + captures.len());
        task_stack.extend(args);
        task_stack.extend(captures);

        let task_id = self.shared.alloc_task_id();

        self.shared.enqueue_task(TaskState {
            id: task_id,
            ip: code_start,
            stack: task_stack,
            call_stack: Vec::new(),
            retain_result,
        });

        if retain_result {
            self.push(Value::Task(task_id))?;
        }
        Ok(())
    }
}
