//! Handler validation helpers for `Std.Tui` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::{Worker, canonical_name, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

const TUI_APPLICATION_HANDLERS_TYPE: &str = "Std.Tui.ApplicationHandlers";

impl Worker {
    /// Validates that `func` is a declared function with the expected `arity`.
    pub(super) fn validate_host_handler_function(
        &self,
        func: &Value,
        arity: u8,
        label: &str,
        help: &'static str,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match func {
            Value::Function { name, .. } => {
                let (_, found_arity) = self
                    .shared
                    .chunk
                    .functions
                    .get(name.as_str())
                    .or_else(|| self.shared.chunk.functions.get(&canonical_name(name)))
                    .copied()
                    .ok_or_else(|| {
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

    /// Pops a `Std.Tui.ApplicationHandlers` record from the stack.
    pub(super) fn pop_tui_application_handlers(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<(String, Value)>, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_APPLICATION_HANDLERS_TYPE => {
                Ok(fields)
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {TUI_APPLICATION_HANDLERS_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a `Std.Tui.ApplicationHandlers` record to `Application.Configure(App, Handlers)`.",
                line,
            )),
        }
    }

    /// Looks up a required field by name (case-insensitive) in a record field list.
    pub(super) fn required_record_field<'a>(
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
                    format!(
                        "Application.Configure(App, Handlers) is missing field `{field_name}`"
                    ),
                    format!(
                        "Build `ApplicationHandlers` with `{field_name} := ...`; malformed bytecode or a broken caller skipped that field."
                    ),
                    line,
                )
            })
    }

    /// Reads an integer field from a record field list.
    pub(super) fn integer_record_field(
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

    /// Reads an optional handler field (`Some(fn)` or `None`) and validates its arity.
    pub(super) fn optional_host_handler_field(
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
