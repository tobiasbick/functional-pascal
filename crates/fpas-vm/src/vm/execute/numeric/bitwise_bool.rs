use super::super::super::diagnostics::VmError;
use super::super::super::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    pub(super) fn try_exec_bitwise_bool(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::BitAnd => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                match (a, b) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a & b))?,
                    (Value::Integer(a), Value::Integer(b)) => self.push(Value::Integer(a & b))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`bitand` requires matching boolean or integer operands",
                            "Use `bitand` with two booleans or two integers.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::BitOr => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                match (a, b) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a | b))?,
                    (Value::Integer(a), Value::Integer(b)) => self.push(Value::Integer(a | b))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`bitor` requires matching boolean or integer operands",
                            "Use `bitor` with two booleans or two integers.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::BitXor => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                match (a, b) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a ^ b))?,
                    (Value::Integer(a), Value::Integer(b)) => self.push(Value::Integer(a ^ b))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`xor` requires matching boolean or integer operands",
                            "Use `xor` with two booleans or two integers.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::EqBool => {
                let b = self.pop_bool(line)?;
                let a = self.pop_bool(line)?;
                self.push(Value::Boolean(a == b))?;
                Ok(true)
            }
            Op::NeqBool => {
                let b = self.pop_bool(line)?;
                let a = self.pop_bool(line)?;
                self.push(Value::Boolean(a != b))?;
                Ok(true)
            }
            Op::Not => {
                let val = self.pop(line)?;
                match val {
                    Value::Boolean(b) => self.push(Value::Boolean(!b))?,
                    Value::Integer(n) => self.push(Value::Integer(!n))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`not` requires boolean or integer",
                            "Use `not` with a boolean or integer value.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::And => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                match (a, b) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a && b))?,
                    (Value::Integer(a), Value::Integer(b)) => self.push(Value::Integer(a & b))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`and` requires matching boolean or integer operands",
                            "Use `and` with two booleans or two integers.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::Or => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                match (a, b) {
                    (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a || b))?,
                    (Value::Integer(a), Value::Integer(b)) => self.push(Value::Integer(a | b))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "`or` requires matching boolean or integer operands",
                            "Use `or` with two booleans or two integers.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
