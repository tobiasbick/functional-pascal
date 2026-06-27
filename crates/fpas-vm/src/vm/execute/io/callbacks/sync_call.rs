use crate::vm::diagnostics::VmError;
use crate::vm::execute::StepResult;
use crate::vm::{CallFrame, Worker, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

impl Worker {
    /// Call a function value synchronously and return its result.
    ///
    /// Uses [`Worker::exec_one`] to step through instructions until the
    /// injected call frame returns, sharing the same dispatch logic as the
    /// main `run()` loop.
    pub(in crate::vm::execute) fn call_function_sync(
        &mut self,
        func: &Value,
        args: &[Value],
        line: SourceLocation,
    ) -> Result<Value, VmError> {
        let Value::Function { name, captures } = func else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected function value, got `{}`", func.type_name()),
                "Pass a named function or a function-typed variable as the callback argument.",
                line,
            ));
        };

        let (code_start, expected_arity) = self.lookup_function_entry(name).ok_or_else(|| {
            runtime_error(
                RUNTIME_UNDEFINED_FUNCTION,
                format!("Undefined function `{name}`"),
                "Declare the function before calling it.",
                line,
            )
        })?;

        if args.len() != expected_arity as usize {
            return Err(runtime_error(
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{name}` expects {expected_arity} arguments, got {}",
                    args.len()
                ),
                "Check the function signature and the number of arguments.",
                line,
            ));
        }

        for arg in args {
            self.push(arg.clone())?;
        }

        let base_slot = self.stack.len() - args.len();
        let saved_depth = self.call_stack.len();
        self.call_stack.push(CallFrame {
            return_ip: self.ip,
            base_slot,
        });
        for capture in captures.clone() {
            self.push(capture)?;
        }
        let saved_ip = self.ip;
        self.ip = code_start;
        self.sync_call_depth += 1;

        let result = self.run_sync_until(saved_depth, line);

        self.sync_call_depth -= 1;
        self.ip = saved_ip;
        result
    }

    pub(in crate::vm::execute::io) fn call_function_sync_allowing_shutdown(
        &mut self,
        func: &Value,
        args: &[Value],
        line: SourceLocation,
    ) -> Result<Value, VmError> {
        let previous_allow_shutdown = self.allow_shutdown_during_sync_call;
        self.allow_shutdown_during_sync_call = true;
        let result = self.call_function_sync(func, args, line);
        self.allow_shutdown_during_sync_call = previous_allow_shutdown;
        result
    }

    /// Step through instructions until the call stack returns to
    /// `target_depth`, i.e. the frame injected by `call_function_sync` has
    /// been popped.
    fn run_sync_until(
        &mut self,
        target_depth: usize,
        caller_line: SourceLocation,
    ) -> Result<Value, VmError> {
        while self.call_stack.len() > target_depth {
            if self.shared.is_shutdown() {
                if self.allow_shutdown_during_sync_call {
                    // Allow cleanup callbacks (for example TUI OnExit) to finish during shutdown.
                } else if self.current_task_id == 0 {
                    return Err(runtime_error(
                        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                        "Execution aborted: a concurrent task failed",
                        "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                        self.current_location,
                    ));
                } else {
                    return Ok(Value::Unit);
                }
            }

            if self.ip >= self.shared.chunk.code().len() {
                return Err(internal_error(
                    "IP ran past end of code during synchronous function call",
                    "This indicates a compiler/runtime bug.",
                    caller_line,
                ));
            }

            match self.exec_one(caller_line)? {
                StepResult::Continue => {}
                StepResult::Halt => break,
                StepResult::Return => {
                    let location = self.current_location;
                    let return_value = self.pop(location)?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_slot);
                        self.push(return_value)?;
                        self.ip = frame.return_ip;
                    }
                }
            }
        }

        self.pop(caller_line)
    }
}
