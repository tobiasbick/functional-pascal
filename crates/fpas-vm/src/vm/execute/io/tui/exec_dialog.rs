//! Turbo Vision modal dialog execution and input-line read-back.
//!
//! **Documentation:** `docs/pascal/std/tui/app/modals.md`

use super::command_map::turbo_vision_command_to_fpas;
use super::tv_geometry::unknown_handle_error;
use super::tv_run::turbo_vision_build_modal_dialog;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionObject, TurboVisionState};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::app::Application as TurboVisionApplication;

impl Worker {
    /// Run a modal Turbo Vision dialog and push `DialogResult`.
    pub(super) fn turbo_vision_exec_dialog(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let dialog_handle = self.pop_turbo_vision_dialog_handle(line)?;
        self.pop_tui_application(line)?;

        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.ExecDialog(App, ...) must run on the main task",
                "Call `Application.ExecDialog` from the main program, not from a `go` task.",
                line,
            ));
        }

        if self.with_tui(|tui| tui.session.is_headless()) {
            let command = self
                .with_tui(|tui| tui.turbo_vision.test_dialog_result.take())
                .unwrap_or(0);
            return self.push_dialog_result(command);
        }

        let mut app = TurboVisionApplication::new().map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Turbo Vision terminal initialization failed: {error}"),
                "Run the program from an interactive terminal or use `Application.OpenForTest` with `Application.TestSetDialogResult` in automated tests.",
                line,
            )
        })?;

        let mut input_bindings = Vec::new();
        let dialog_view = self.with_tui(|tui| {
            turbo_vision_build_modal_dialog(
                &tui.turbo_vision.objects,
                dialog_handle,
                &mut input_bindings,
            )
        });
        let Some(mut dialog_view) = dialog_view else {
            return Err(unknown_handle_error("Dialog", dialog_handle, line));
        };

        let command = i64::from(turbo_vision_command_to_fpas(dialog_view.execute(&mut app)));
        self.with_tui(|tui| {
            for (child_handle, binding) in &input_bindings {
                if let Some(TurboVisionObject::InputLine(input_line)) =
                    tui.turbo_vision.objects.get(child_handle)
                {
                    input_line.text_cell.commit_view_binding(binding);
                }
            }
        });
        self.push_dialog_result(command)
    }

    /// Read the current text of an `InputLine` handle.
    pub(super) fn turbo_vision_input_text(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let input_line_handle = self.pop_turbo_vision_input_line_handle(line)?;
        self.pop_tui_application(line)?;

        let text =
            self.with_tui(|tui| input_line_text(&tui.turbo_vision, input_line_handle, line))?;
        self.push(Value::Str(text))
    }

    /// Read the checked state of a `CheckBox` handle.
    pub(super) fn turbo_vision_checked(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let check_box_handle = self.pop_turbo_vision_check_box_handle(line)?;
        self.pop_tui_application(line)?;

        let checked =
            self.with_tui(|tui| check_box_checked(&tui.turbo_vision, check_box_handle, line))?;
        self.push(Value::Boolean(checked))
    }

    /// Read the selected state of a `RadioButton` handle.
    pub(super) fn turbo_vision_selected(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let radio_button_handle = self.pop_turbo_vision_radio_button_handle(line)?;
        self.pop_tui_application(line)?;

        let selected = self
            .with_tui(|tui| radio_button_selected(&tui.turbo_vision, radio_button_handle, line))?;
        self.push(Value::Boolean(selected))
    }

    /// Queue the closing command consumed by the next headless `Application.ExecDialog` call.
    pub(super) fn turbo_vision_test_set_dialog_result(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let command = self.pop_int(line)?;
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            tui.turbo_vision.test_dialog_result = Some(command);
        });
        Ok(())
    }
}

fn input_line_text(
    state: &TurboVisionState,
    handle: u32,
    line: SourceLocation,
) -> Result<String, VmError> {
    match state.objects.get(&handle) {
        Some(TurboVisionObject::InputLine(input_line)) => Ok(input_line.text_cell.read()),
        _ => Err(unknown_handle_error("InputLine", handle, line)),
    }
}

fn check_box_checked(
    state: &TurboVisionState,
    handle: u32,
    line: SourceLocation,
) -> Result<bool, VmError> {
    match state.objects.get(&handle) {
        Some(TurboVisionObject::CheckBox(check_box)) => Ok(check_box.checked_cell.read()),
        _ => Err(unknown_handle_error("CheckBox", handle, line)),
    }
}

fn radio_button_selected(
    state: &TurboVisionState,
    handle: u32,
    line: SourceLocation,
) -> Result<bool, VmError> {
    match state.objects.get(&handle) {
        Some(TurboVisionObject::RadioButton(radio_button)) => Ok(radio_button.selected_cell.read()),
        _ => Err(unknown_handle_error("RadioButton", handle, line)),
    }
}
