//! `Std.Tui` application session lifecycle helpers.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use crate::vm::shared::TuiState;
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";

impl Worker {
    /// Acquires the TUI state lock for the duration of `f`.
    pub(in crate::vm::execute::io) fn with_tui<R>(&self, f: impl FnOnce(&mut TuiState) -> R) -> R {
        f(&mut self.shared.tui.lock().unwrap_or_else(|e| e.into_inner()))
    }

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

    /// Reads an optional handler function from an `ApplicationHandlers` record field.
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

    /// Pops a handler function and an `Application` record, validates arity, then stores it.
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

    /// Invokes the registered Turbo Vision `OnCommand` handler.
    pub(in crate::vm::execute::io) fn dispatch_tui_command(
        &mut self,
        command: fpas_std::CommandEvent,
        line: SourceLocation,
    ) -> Result<fpas_std::ProcessOutcome, VmError> {
        use fpas_std::ProcessOutcome;

        let handler = self.with_tui(|tui| tui.on_command.clone());
        let Some(handler) = handler else {
            return Ok(ProcessOutcome::Command { handled: false });
        };

        let app_rec = Self::tui_application_record();
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Value::Integer(command.id.0)],
            line,
        )?;
        Ok(ProcessOutcome::Command { handled: true })
    }

    /// Pops a `Std.Tui.Application` record from the stack, returning an error on type mismatch.
    pub(in crate::vm::execute::io) fn pop_tui_application(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match self.pop(line)? {
            Value::Record { type_name, .. } if type_name == TUI_APPLICATION_TYPE => Ok(()),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {TUI_APPLICATION_TYPE}, got {}", other.type_name()),
                "Pass the value returned by Std.Tui.Application.Open().",
                line,
            )),
        }
    }

    /// Closes the TUI session and resets Turbo Vision state.
    pub(in crate::vm::execute::io) fn close_tui_application_state(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.turbo_vision_shutdown_live_app();
        self.turbo_vision_shutdown_headless_app();
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let close_result = self.with_console_and_key_input(|console, key_input| {
            tui.session.close(console, key_input, line)
        });
        tui.turbo_vision_on_key = None;
        tui.turbo_vision_on_mouse = None;
        tui.on_command = None;
        tui.quit_requested = false;
        tui.turbo_vision = Default::default();
        close_result?;
        Ok(())
    }

    /// Clears Turbo Vision state before opening a new application session.
    pub(in crate::vm::execute::io) fn reset_tui_session_state(&mut self) {
        self.turbo_vision_shutdown_live_app();
        self.turbo_vision_shutdown_headless_app();
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.turbo_vision_on_key = None;
        tui.turbo_vision_on_mouse = None;
        tui.on_command = None;
        tui.quit_requested = false;
        tui.turbo_vision = Default::default();
    }
}
