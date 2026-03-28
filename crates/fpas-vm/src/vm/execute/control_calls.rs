use super::super::diagnostics::VmError;
use super::super::{CallFrame, Worker, runtime_error};
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

        let base = self.stack.len() - argc as usize;
        self.call_stack.push(CallFrame {
            return_ip: self.ip,
            base_slot: base,
        });
        self.ip = code_start;
        Ok(())
    }
}
