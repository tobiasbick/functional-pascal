//! Try-2 VM intrinsic handlers (`Dialog.NewModal`, `Application.ExecView`, …).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::headless::try2_ensure_headless_app;
use super::modals::try2_exec_view;
use super::records::TUI_DIALOG_TYPE;
use super::views::{try2_dialog_add_button, try2_dialog_new_modal};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::handle_records::HANDLE_FIELD;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

impl Worker {
    /// Dispatches try-2 TUI intrinsics.
    pub(in crate::vm::execute::io::tui) fn try_exec_try2_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let Intrinsic::Tui(code) = intrinsic else {
            return Ok(false);
        };

        match code {
            TuiIntrinsic::DialogNewModal => {
                let title = self.pop_turbo_vision_string("Dialog title", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_dialog_new_modal(self, bounds, title, line)?;
                self.push(Self::turbo_vision_dialog_record(handle))?;
            }
            TuiIntrinsic::DialogAddButton => {
                let is_default = self.pop_bool(line)?;
                let command = self.pop_int(line)?;
                let command = u16::try_from(command).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Button command id is outside the Turbo Vision u16 range",
                        "Use a command id from 0 to 65535.",
                        line,
                    )
                })?;
                let text = self.pop_turbo_vision_string("Button text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let dialog_handle = self.pop_try2_handle(TUI_DIALOG_TYPE, "Dialog", line)?;
                let button_handle = try2_dialog_add_button(
                    self,
                    dialog_handle,
                    bounds,
                    text,
                    command,
                    is_default,
                    line,
                )?;
                self.push(Self::turbo_vision_button_record(button_handle))?;
            }
            TuiIntrinsic::ExecView => {
                let dialog_handle = self.pop_try2_handle(TUI_DIALOG_TYPE, "Dialog", line)?;
                self.pop_tui_application(line)?;
                let command = try2_exec_view(self, dialog_handle, line)?;
                self.push(Value::Integer(i64::from(command)))?;
            }
            TuiIntrinsic::Try2InjectKeyboard => {
                let key_code = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let key_code = u16::try_from(key_code).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("Keyboard key code {key_code} is outside the u16 range"),
                        "Pass a turbo-vision key code such as KB_ENTER (0x1C0D).",
                        line,
                    )
                })?;
                self.try2_inject_keyboard(key_code, line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn pop_try2_handle(
        &mut self,
        expected_type: &str,
        label: &str,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == expected_type => {
                self.decode_try2_handle(&fields, label, line)
            }
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected {expected_type}, got {}", other.type_name()),
                format!("Pass a {label} handle from the try-2 constructor."),
                line,
            )),
        }
    }

    fn decode_try2_handle(
        &self,
        fields: &[(String, Value)],
        label: &str,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        let handle = fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(HANDLE_FIELD))
            .and_then(|(_, value)| match value {
                Value::Integer(id) if *id >= 0 => Some(*id as u32),
                _ => None,
            })
            .ok_or_else(|| {
                runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("{label} handle record is missing `{HANDLE_FIELD}`"),
                    format!("Use a handle returned by the try-2 {label} constructor."),
                    line,
                )
            })?;
        Ok(handle)
    }

    fn try2_inject_keyboard(
        &mut self,
        key_code: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Try2InjectKeyboard is only supported in headless OpenForTest sessions",
                "Call `Application.OpenForTest` before injecting synthetic keys.",
                line,
            ));
        }
        try2_ensure_headless_app(self, line)?;
        if let Some(app) = self.headless_tv_app.as_ref() {
            app.push_keyboard(key_code);
        }
        Ok(())
    }
}
