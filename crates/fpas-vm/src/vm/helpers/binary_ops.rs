use crate::vm::diagnostics::TYPE_MISMATCH_CODE;
use crate::vm::{VmError, Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};

impl Worker {
    pub(in crate::vm) fn binary_int(
        &mut self,
        location: SourceLocation,
        f: impl FnOnce(i64, i64) -> Result<Value, VmError>,
    ) -> Result<(), VmError> {
        let right = self.pop(location)?;
        let left = self.pop(location)?;

        let to_i64 = |value: &Value| -> Option<i64> {
            match value {
                Value::Integer(number) => Some(*number),
                Value::Char(ch) => Some(*ch as i64),
                Value::Boolean(flag) => Some(if *flag { 1 } else { 0 }),
                _ => None,
            }
        };

        match (to_i64(&left), to_i64(&right)) {
            (Some(left), Some(right)) => {
                let result = f(left, right)?;
                self.push(result)
            }
            _ => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "Integer operation requires integer operands",
                "Use integer-compatible operands (integer, char, boolean) for this operation.",
                location,
            )),
        }
    }

    pub(in crate::vm) fn binary_real(
        &mut self,
        location: SourceLocation,
        f: impl FnOnce(f64, f64) -> Result<Value, VmError>,
    ) -> Result<(), VmError> {
        let right = self.pop(location)?;
        let left = self.pop(location)?;
        match (left, right) {
            (Value::Real(left), Value::Real(right)) => {
                let result = f(left, right)?;
                self.push(result)
            }
            _ => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "Real operation requires real operands",
                "Use real operands for this operation.",
                location,
            )),
        }
    }

    pub(in crate::vm) fn binary_str(
        &mut self,
        location: SourceLocation,
        f: impl FnOnce(&str, &str) -> Value,
    ) -> Result<(), VmError> {
        let right = self.pop(location)?;
        let left = self.pop(location)?;
        match (&left, &right) {
            (Value::Str(left), Value::Str(right)) => self.push(f(left, right)),
            (Value::Char(ch), Value::Str(right)) => {
                let left = ch.to_string();
                self.push(f(&left, right))
            }
            (Value::Str(left), Value::Char(ch)) => {
                let right = ch.to_string();
                self.push(f(left, &right))
            }
            (Value::Char(left), Value::Char(right)) => {
                let left = left.to_string();
                let right = right.to_string();
                self.push(f(&left, &right))
            }
            _ => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "String operation requires string operands",
                "Use string operands for this operation.",
                location,
            )),
        }
    }
}
