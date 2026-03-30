use super::super::diagnostics::VmError;
use super::super::{CallFrame, Worker, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

impl Worker {
    pub(super) fn try_exec_control_calls(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::Jump(addr) => {
                self.ip = addr as usize;
                Ok(true)
            }
            Op::JumpIfFalse(addr) => {
                let val = self.pop(line)?;
                if !self.is_truthy(&val) {
                    self.ip = addr as usize;
                }
                Ok(true)
            }
            Op::JumpIfTrue(addr) => {
                let val = self.pop(line)?;
                if self.is_truthy(&val) {
                    self.ip = addr as usize;
                }
                Ok(true)
            }
            Op::Call(name_idx, argc) => {
                let name = self.const_str(name_idx, line)?;
                self.call_named_function(&name, argc, line)?;
                Ok(true)
            }
            Op::CallVirtual(method_name_idx, argc) => {
                let method_name = self.const_str(method_name_idx, line)?;
                let qualified = self.resolve_virtual_method_name(&method_name, argc, line)?;
                self.call_named_function(&qualified, argc, line)?;
                Ok(true)
            }
            Op::CallValue(argc) => {
                let func = self.pop(line)?;
                let (name, captures) = match func {
                    Value::Function { name, captures } => (name, captures),
                    other => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            format!("Expected function value, got `{}`", other.type_name()),
                            "Only function values can be called with CallValue. Check that the variable holds a function.",
                            line,
                        ));
                    }
                };
                self.call_named_function(&name, argc, line)?;
                // Push captured values after args (accessible as locals after params).
                for cap in captures {
                    self.push(cap)?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Resolve `TypeName.method_name` from the receiver at the bottom of the current arg window.
    ///
    /// The receiver is the first of `argc` arguments: `stack[sp - argc]`.
    fn resolve_virtual_method_name(
        &self,
        method_name: &str,
        argc: u8,
        line: SourceLocation,
    ) -> Result<String, VmError> {
        let sp = self.stack.len();
        let receiver_idx = sp.checked_sub(argc as usize).ok_or_else(|| {
            internal_error(
                format!("CallVirtual expected {argc} argument(s) on the stack, found {sp}"),
                "This indicates invalid bytecode or a VM stack-layout bug. Please report it.",
                line,
            )
        })?;
        let receiver = self.stack.get(receiver_idx).ok_or_else(|| {
            runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "CallVirtual: stack underflow — no receiver",
                "Ensure the receiver is on the stack before arguments.",
                line,
            )
        })?;

        let type_name = match receiver {
            Value::Record { type_name, .. } => type_name.clone(),
            Value::Ref { type_name, .. } => type_name.clone(),
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "CallVirtual: receiver must be a record or ref, got `{}`",
                        other.type_name()
                    ),
                    "Only record values can be the receiver of an interface method call.",
                    line,
                ));
            }
        };

        Ok(format!("{type_name}.{method_name}"))
    }

    fn call_named_function(
        &mut self,
        name: &str,
        argc: u8,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let (code_start, expected_arity) = self
            .shared
            .chunk
            .functions
            .get(name)
            .copied()
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_UNDEFINED_FUNCTION,
                    format!("Undefined function `{name}`"),
                    "Declare the function before calling it, or fix the function name.",
                    line,
                )
            })?;

        if argc != expected_arity {
            return Err(runtime_error(
                RUNTIME_WRONG_CALL_ARITY,
                format!("Function `{name}` expects {expected_arity} arguments, got {argc}"),
                "Call the function with the declared number of arguments.",
                line,
            ));
        }

        if self.stack.len() < argc as usize {
            return Err(internal_error(
                format!(
                    "Call to `{name}` expected {argc} argument(s) on the stack, found {}",
                    self.stack.len()
                ),
                "This indicates invalid bytecode or a VM stack-layout bug. Please report it.",
                line,
            ));
        }

        let base = self.stack.len() - argc as usize;
        self.call_stack.push(CallFrame {
            return_ip: self.ip,
            base_slot: base,
        });
        self.ip = code_start;
        Ok(())
    }
}
