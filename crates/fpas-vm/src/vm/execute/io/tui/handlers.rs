//! Handler validation helpers for `Std.Tui` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app.md` (from the repository root).

use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use crate::vm::shared::TuiState;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

const TUI_APPLICATION_HANDLERS_TYPE: &str = "Std.Tui.ApplicationHandlers";

impl Worker {
    /// Validates that `func` is a declared function with the expected `arity`.
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

    /// Reads an optional handler field (`Some(fn)` or `None`) and validates its arity.
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

    /// Reads an optional integer field from the stack.
    pub(in crate::vm::execute::io) fn pop_optional_integer(
        &mut self,
        label: &str,
        line: SourceLocation,
    ) -> Result<Option<i64>, VmError> {
        match self.pop(line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => match *inner {
                Value::Integer(value) => Ok(Some(value)),
                other => Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "{label} expects `Option of integer`, got Some({})",
                        other.type_name()
                    ),
                    "Pass `None` or `Some(<color index>)` using a CRT color constant or integer from 0 to 15.",
                    line,
                )),
            },
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "{label} expects `Option of integer`, got {}",
                    other.type_name()
                ),
                "Pass `None` or `Some(<color index>)` using a CRT color constant or integer from 0 to 15.",
                line,
            )),
        }
    }

    /// Reads an optional character field from the stack.
    pub(in crate::vm::execute::io) fn pop_optional_char(
        &mut self,
        label: &str,
        line: SourceLocation,
    ) -> Result<Option<char>, VmError> {
        match self.pop(line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => match *inner {
                Value::Char(value) => Ok(Some(value)),
                other => Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "{label} expects `Option of char`, got Some({})",
                        other.type_name()
                    ),
                    "Pass `None` or `Some('.')` with a single character literal.",
                    line,
                )),
            },
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "{label} expects `Option of char`, got {}",
                    other.type_name()
                ),
                "Pass `None` or `Some('.')` with a single character literal.",
                line,
            )),
        }
    }

    /// Acquires the TUI state lock for the duration of `f`.
    ///
    /// Prefer this over bare `.lock().unwrap_or_else(...)` for simple reads/writes that do **not**
    /// need to call other `&mut self` methods while the lock is held.
    pub(in crate::vm::execute::io) fn with_tui<R>(&self, f: impl FnOnce(&mut TuiState) -> R) -> R {
        f(&mut self.shared.tui.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// Pops a handler function and an `Application` record from the stack, validates arity, then
    /// stores the handler via `setter`.  Eliminates boilerplate from every `TuiHostRegister*` arm.
    pub(super) fn register_tui_handler(
        &mut self,
        arity: u8,
        label: &'static str,
        hint: &'static str,
        setter: impl FnOnce(&mut TuiState, Value),
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let func = self.pop(line)?;
        self.pop_tui_application(line)?;
        self.validate_host_handler_function(&func, arity, label, hint, line)?;
        self.with_tui(|tui| setter(tui, func));
        Ok(())
    }
}
