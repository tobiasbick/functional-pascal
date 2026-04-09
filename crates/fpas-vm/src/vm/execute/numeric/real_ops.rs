use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_DIVISION_BY_ZERO;

impl Worker {
    pub(super) fn try_exec_real_ops(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::AddReal => {
                self.binary_real(line, |a, b| Ok(Value::Real(a + b)))?;
                Ok(true)
            }
            Op::SubReal => {
                self.binary_real(line, |a, b| Ok(Value::Real(a - b)))?;
                Ok(true)
            }
            Op::MulReal => {
                self.binary_real(line, |a, b| Ok(Value::Real(a * b)))?;
                Ok(true)
            }
            Op::DivReal => {
                self.binary_real(line, |a, b| {
                    if b == 0.0 {
                        Err(runtime_error(
                            RUNTIME_DIVISION_BY_ZERO,
                            "Division by zero",
                            "Check the right-hand side before using `/`.",
                            line,
                        ))
                    } else {
                        Ok(Value::Real(a / b))
                    }
                })?;
                Ok(true)
            }
            Op::NegateReal => {
                let n = self.pop_real(line)?;
                self.push(Value::Real(-n))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
