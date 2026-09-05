//! Array construction and consuming mutations.
//! Documentation: `docs/pascal/std/collections/array/mutating.md`.

use super::*;

impl Worker {
    /// Constructs an array from the instruction register window.
    pub fn make_array(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let values = self.window(o.b, usize::from(o.c))?;
        self.write(register(o.a)?, Value::Array(values.into()))
    }

    /// Consumes the source array and appends with copy-on-write isolation.
    pub fn array_push(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let value = self.read(register(o.c)?)?.clone();
        let array = self.take(register(o.b)?)?;
        let Value::Array(mut array) = array else {
            return Err(self.type_mismatch("array", &array));
        };
        array.push(value);
        self.write(register(o.a)?, Value::Array(array))
    }

    /// Removes the last element while preserving shared arrays and invalid operands.
    pub fn array_pop(&mut self, o: AbcOperands) -> Result<(), VmError> {
        let source = register(o.b)?;
        match self.read(source)? {
            Value::Array(array) if !array.is_empty() => {}
            Value::Array(_) => {
                return Err(self.aggregate_error_code(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    "Array index -1 out of bounds (len 0)",
                    "Check array length before calling Std.Array.Pop.",
                ));
            }
            other => return Err(self.type_mismatch("array", other)),
        }
        let Value::Array(mut array) = self.take(source)? else {
            return Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Validated ArrayPop source changed type before commit",
            ));
        };
        let value = array.pop().ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Validated ArrayPop source became empty before commit",
            )
        })?;
        self.write(source, Value::Array(array))?;
        self.write(register(o.a)?, value)
    }
}
