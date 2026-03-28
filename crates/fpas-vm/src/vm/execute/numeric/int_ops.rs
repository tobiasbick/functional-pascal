use super::super::super::{Vm, VmError, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

impl Vm {
    pub(super) fn try_exec_int_ops(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::AddInt => {
                self.binary_int(line, |a, b| Ok(Value::Integer(a.wrapping_add(b))))?;
                Ok(true)
            }
            Op::SubInt => {
                self.binary_int(line, |a, b| Ok(Value::Integer(a.wrapping_sub(b))))?;
                Ok(true)
            }
            Op::MulInt => {
                self.binary_int(line, |a, b| Ok(Value::Integer(a.wrapping_mul(b))))?;
                Ok(true)
            }
            Op::DivInt => {
                self.binary_int(line, |a, b| {
                    if b == 0 {
                        Err(runtime_error(
                            RUNTIME_DIVISION_BY_ZERO,
                            "Division by zero",
                            "Check the right-hand side before using `div` or `/`.",
                            line,
                        ))
                    } else {
                        Ok(Value::Integer(a / b))
                    }
                })?;
                Ok(true)
            }
            Op::ModInt => {
                self.binary_int(line, |a, b| {
                    if b == 0 {
                        Err(runtime_error(
                            RUNTIME_MODULO_BY_ZERO,
                            "Modulo by zero",
                            "Check the right-hand side before using `mod`.",
                            line,
                        ))
                    } else {
                        Ok(Value::Integer(a % b))
                    }
                })?;
                Ok(true)
            }
            Op::NegateInt => {
                let val = self.pop(line)?;
                match val {
                    Value::Integer(n) => self.push(Value::Integer(-n))?,
                    Value::Real(n) => self.push(Value::Real(-n))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "Cannot negate non-numeric value",
                            "Apply unary `-` only to integer or real values.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::Shl => {
                self.binary_int(line, |a, b| Ok(Value::Integer(a << (b as u32))))?;
                Ok(true)
            }
            Op::Shr => {
                self.binary_int(line, |a, b| Ok(Value::Integer(a >> (b as u32))))?;
                Ok(true)
            }
            Op::IntToReal => {
                let n = self.pop_int(line)?;
                self.push(Value::Real(n as f64))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
