use super::super::super::super::diagnostics::VmError;
use super::super::super::super::{CallFrame, Worker, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_PROGRAM_PANIC, RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
    RUNTIME_WRONG_CALL_ARITY,
};

impl Worker {
    /// Call a function value synchronously and return its result.
    ///
    /// Uses the existing run loop by pushing a call frame and stepping
    /// until that frame returns.
    pub(super) fn call_function_sync(
        &mut self,
        func: &Value,
        args: &[Value],
        line: SourceLocation,
    ) -> Result<Value, VmError> {
        let (name, captures) = match func {
            Value::Function { name, captures } => (name.clone(), captures.clone()),
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Expected function value, got `{}`", other.type_name()),
                    "Pass a function (named or anonymous) as the callback argument.",
                    line,
                ));
            }
        };

        let (code_start, expected_arity) = self
            .shared
            .chunk
            .functions
            .get(&name)
            .copied()
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_UNDEFINED_FUNCTION,
                    format!("Undefined function `{name}`"),
                    "Declare the function before calling it.",
                    line,
                )
            })?;

        if args.len() as u8 != expected_arity {
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
        for capture in captures {
            self.push(capture)?;
        }
        let saved_ip = self.ip;
        self.ip = code_start;

        while self.call_stack.len() > saved_depth {
            if self.ip >= self.shared.chunk.code.len() {
                return Err(internal_error(
                    "IP ran past end of code during synchronous function call",
                    "This indicates a compiler/runtime bug.",
                    line,
                ));
            }

            let op = self.shared.chunk.code[self.ip];
            let location = self.shared.chunk.location_at(self.ip).unwrap_or(line);
            self.current_location = location;
            self.ip += 1;

            if self.try_exec_stack_scope(op, location)?
                || self.try_exec_numeric(op, location)?
                || self.try_exec_control_calls(op, location)?
                || self.try_exec_aggregates(op, location)?
                || self.try_exec_result_option(op, location)?
                || self.try_exec_io(op, location)?
            {
                continue;
            }

            match op {
                Op::Return => {
                    let return_value = self.pop(location)?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_slot);
                        self.push(return_value)?;
                        self.ip = frame.return_ip;
                    }
                }
                Op::Halt => break,
                Op::Panic => {
                    let value = self.pop(location)?;
                    return Err(runtime_error(
                        RUNTIME_PROGRAM_PANIC,
                        format!("panic: {value}"),
                        "Remove the panic or guard the failing condition.",
                        location,
                    ));
                }
                _ => {
                    return Err(internal_error(
                        format!("Unhandled opcode in sync call: {op:?}"),
                        "This indicates a VM dispatch bug.",
                        location,
                    ));
                }
            }
        }

        self.ip = saved_ip;
        self.pop(line)
    }
}
