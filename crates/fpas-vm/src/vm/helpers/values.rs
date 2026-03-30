use super::super::diagnostics::TYPE_MISMATCH_CODE;
use super::super::{VmError, Worker, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};

impl Worker {
    pub(in super::super) fn const_value(
        &self,
        idx: u16,
        location: SourceLocation,
    ) -> Result<&Value, VmError> {
        self.shared
            .chunk
            .constants
            .get(idx as usize)
            .ok_or_else(|| {
                internal_error(
                    format!(
                        "constant index {idx} out of bounds (len {})",
                        self.shared.chunk.constants.len()
                    ),
                    "This indicates invalid bytecode or a compiler constant-pool bug. Please report it.",
                    location,
                )
            })
    }

    pub(in super::super) fn pop_int(&mut self, location: SourceLocation) -> Result<i64, VmError> {
        match self.pop(location)? {
            Value::Integer(value) => Ok(value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected integer, got {}", other.type_name()),
                "Use integer operands for this VM operation.",
                location,
            )),
        }
    }

    pub(in super::super) fn pop_real(&mut self, location: SourceLocation) -> Result<f64, VmError> {
        match self.pop(location)? {
            Value::Real(value) => Ok(value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected real, got {}", other.type_name()),
                "Use real operands for this VM operation.",
                location,
            )),
        }
    }

    pub(in super::super) fn pop_bool(&mut self, location: SourceLocation) -> Result<bool, VmError> {
        match self.pop(location)? {
            Value::Boolean(value) => Ok(value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected boolean, got {}", other.type_name()),
                "Use boolean operands for this VM operation.",
                location,
            )),
        }
    }

    pub(in super::super) fn const_str(
        &self,
        idx: u16,
        location: SourceLocation,
    ) -> Result<String, VmError> {
        match self.const_value(idx, location)? {
            Value::Str(value) => Ok(value.clone()),
            _ => Err(internal_error(
                "Expected string constant",
                "This indicates a compiler constant-pool bug. Please report it.",
                location,
            )),
        }
    }
}
