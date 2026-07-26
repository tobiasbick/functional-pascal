use crate::vm::diagnostics::TYPE_MISMATCH_CODE;
use crate::vm::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};

impl crate::vm::Worker {
    pub(in crate::vm) fn binary_int(
        &mut self,
        location: SourceLocation,
        f: impl FnOnce(i64, i64) -> Result<Value, VmError>,
    ) -> Result<(), VmError> {
        let stack_len = self.stack.len();
        if stack_len < 2 {
            self.pop(location)?;
            return self.pop(location).map(drop);
        }
        let left_index = stack_len - 2;

        // Hot path: typed integer ops almost always see Integer×Integer.
        let to_i64 = |value: &Value| -> Option<i64> {
            match value {
                Value::Integer(number) => Some(*number),
                Value::Boolean(flag) => Some(if *flag { 1 } else { 0 }),
                _ => None,
            }
        };

        let result = match (&self.stack[left_index], &self.stack[left_index + 1]) {
            (Value::Integer(left), Value::Integer(right)) => f(*left, *right),
            (left, right) => match (to_i64(left), to_i64(right)) {
                (Some(left), Some(right)) => f(left, right),
                _ => Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    "Integer operation requires integer operands",
                    "Use integer-compatible operands (integer, boolean) for this operation.",
                    location,
                )),
            },
        };

        match result {
            Ok(result) => {
                self.stack[left_index] = result;
                self.stack.pop();
                Ok(())
            }
            Err(error) => {
                self.stack.truncate(left_index);
                Err(error)
            }
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
            _ => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                "String operation requires string operands",
                "Use string operands for this operation.",
                location,
            )),
        }
    }
}
