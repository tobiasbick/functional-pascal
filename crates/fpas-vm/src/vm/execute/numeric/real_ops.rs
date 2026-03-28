use super::super::super::Worker;
use super::super::super::diagnostics::VmError;
use fpas_bytecode::{Op, SourceLocation, Value};

impl Worker {
    pub(super) fn try_exec_real_ops(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::AddReal => {
                self.binary_real(line, |a, b| Value::Real(a + b))?;
                Ok(true)
            }
            Op::SubReal => {
                self.binary_real(line, |a, b| Value::Real(a - b))?;
                Ok(true)
            }
            Op::MulReal => {
                self.binary_real(line, |a, b| Value::Real(a * b))?;
                Ok(true)
            }
            Op::DivReal => {
                self.binary_real(line, |a, b| Value::Real(a / b))?;
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
