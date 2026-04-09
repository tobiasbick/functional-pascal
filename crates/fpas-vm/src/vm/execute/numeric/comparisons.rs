use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    pub(super) fn try_exec_comparisons(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::ConcatStr => {
                let b = self.pop(line)?;
                let a = self.pop(line)?;
                let sa = match a {
                    Value::Str(s) => s,
                    Value::Char(c) => c.to_string(),
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "ConcatStr requires two strings",
                            "Use string operands when concatenating values.",
                            line,
                        ));
                    }
                };
                let sb = match b {
                    Value::Str(s) => s,
                    Value::Char(c) => c.to_string(),
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "ConcatStr requires two strings",
                            "Use string operands when concatenating values.",
                            line,
                        ));
                    }
                };
                self.push(Value::Str(sa + &sb))?;
                Ok(true)
            }
            Op::EqInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a == b)))?;
                Ok(true)
            }
            Op::NeqInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a != b)))?;
                Ok(true)
            }
            Op::LtInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a < b)))?;
                Ok(true)
            }
            Op::GtInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a > b)))?;
                Ok(true)
            }
            Op::LeInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a <= b)))?;
                Ok(true)
            }
            Op::GeInt => {
                self.binary_int(line, |a, b| Ok(Value::Boolean(a >= b)))?;
                Ok(true)
            }
            Op::EqReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a == b)))?;
                Ok(true)
            }
            Op::NeqReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a != b)))?;
                Ok(true)
            }
            Op::LtReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a < b)))?;
                Ok(true)
            }
            Op::GtReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a > b)))?;
                Ok(true)
            }
            Op::LeReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a <= b)))?;
                Ok(true)
            }
            Op::GeReal => {
                self.binary_real(line, |a, b| Ok(Value::Boolean(a >= b)))?;
                Ok(true)
            }
            Op::EqStr => {
                self.binary_str(line, |a, b| Value::Boolean(a == b))?;
                Ok(true)
            }
            Op::NeqStr => {
                self.binary_str(line, |a, b| Value::Boolean(a != b))?;
                Ok(true)
            }
            Op::LtStr => {
                self.binary_str(line, |a, b| Value::Boolean(a < b))?;
                Ok(true)
            }
            Op::GtStr => {
                self.binary_str(line, |a, b| Value::Boolean(a > b))?;
                Ok(true)
            }
            Op::LeStr => {
                self.binary_str(line, |a, b| Value::Boolean(a <= b))?;
                Ok(true)
            }
            Op::GeStr => {
                self.binary_str(line, |a, b| Value::Boolean(a >= b))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
