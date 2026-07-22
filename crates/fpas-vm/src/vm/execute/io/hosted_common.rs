//! Shared helpers for hosted `Application.Run` loops.
//!
//! **Documentation:** `docs/pascal/std/graph/app/README.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

/// Builds an exit-reason enum value for hosted application run loops.
pub(in crate::vm::execute::io) fn hosted_exit_reason(type_name: &str, variant: &str) -> Value {
    Value::Enum {
        type_name: type_name.into(),
        variant: variant.into(),
        fields: vec![],
    }
}

impl Worker {
    /// Validates that a hosted callback is a declared function with the expected arity.
    pub(in crate::vm::execute::io) fn validate_host_handler_function(
        &self,
        func: &Value,
        arity: u8,
        label: &str,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match func {
            Value::Function { name, .. } => {
                let (_, found_arity) = self.lookup_function_entry(name).ok_or_else(|| {
                    runtime_error(
                        RUNTIME_UNDEFINED_FUNCTION,
                        format!("Undefined function `{name}` for {label}"),
                        "Declare the handler before registering it.",
                        line,
                    )
                })?;
                if found_arity != arity {
                    return Err(runtime_error(
                        RUNTIME_WRONG_CALL_ARITY,
                        format!("{label} handler must have arity {arity}, got {found_arity}"),
                        help,
                        line,
                    ));
                }
                Ok(())
            }
            _ => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("{label} expects a function value"),
                help,
                line,
            )),
        }
    }

    /// Returns a required record field, matched case-insensitively.
    pub(in crate::vm::execute::io) fn required_record_field<'a>(
        fields: &'a [(String, Value)],
        field_name: &str,
        line: SourceLocation,
    ) -> Result<&'a Value, VmError> {
        fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(field_name))
            .map(|(_, value)| value)
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Application.Configure(App, Handlers) is missing field `{field_name}`"),
                    format!(
                        "Build `ApplicationHandlers` with `{field_name} := ...`; malformed bytecode or a broken caller skipped that field."
                    ),
                    line,
                )
            })
    }

    /// Reads an integer field from a hosted handler record.
    pub(in crate::vm::execute::io) fn integer_record_field(
        &self,
        fields: &[(String, Value)],
        field_name: &str,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        match Self::required_record_field(fields, field_name, line)? {
            Value::Integer(value) => Ok(*value),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "ApplicationHandlers.{field_name} must be integer, got {}",
                    other.type_name()
                ),
                format!(
                    "Set `{field_name} := <milliseconds>` with an integer value in the handler bundle."
                ),
                line,
            )),
        }
    }

    /// Reads and validates an optional hosted handler from a record field.
    pub(in crate::vm::execute::io) fn optional_host_handler_field(
        &self,
        fields: &[(String, Value)],
        field_name: &str,
        arity: u8,
        label: &str,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        match Self::required_record_field(fields, field_name, line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => {
                self.validate_host_handler_function(inner, arity, label, help, line)?;
                Ok(Some((**inner).clone()))
            }
            _ => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("ApplicationHandlers.{field_name} must be `Some(handler)` or `None`"),
                help,
                line,
            )),
        }
    }
}
