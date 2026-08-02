use crate::vm::diagnostics::VmError;
use crate::vm::execute::transition::{ExecutionContext, ExecutionTransition};
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
        let Value::Function(function) = func else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected function value, got `{}`", func.type_name()),
                "Pass a named function or a function-typed variable as the callback argument.",
                line,
            ));
        };

        let (code_start, expected_arity) =
            self.lookup_function_entry(&function.name).ok_or_else(|| {
                runtime_error(
                    RUNTIME_UNDEFINED_FUNCTION,
                    format!("Undefined function `{}`", function.name),
                    "Declare the function before calling it.",
                    line,
                )
            })?;

        if args.len() != expected_arity as usize {
            return Err(runtime_error(
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {expected_arity} arguments, got {}",
                    function.name,
                    args.len()
                ),
                "Check the function signature and the number of arguments.",
                line,
            ));
        }

        self.validate_code_entry(code_start, line)?;

        for arg in args {
            self.push(arg.clone())?;
        }

        let base_slot = self.stack.len() - args.len();
        let saved_depth = self.call_stack.len();
        self.push_call_frame(
            CallFrame {
                return_ip: self.ip,
                base_slot,
            },
            line,
        )?;
        for capture in &function.captures {
            self.push(capture.clone())?;
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
            match self.advance_execution(ExecutionContext::SynchronousCallback, caller_line)? {
                ExecutionTransition::Continue => {}
                ExecutionTransition::Cancelled => return Ok(Value::Unit),
                ExecutionTransition::Completed(_) | ExecutionTransition::Suspended => {
                    return Err(internal_error(
                        "Synchronous callback produced an invalid terminal transition",
                        "This indicates a VM callback execution bug. Please report it.",
                        caller_line,
                    ));
                }
            }
        }

        self.pop(caller_line)
    }
}
