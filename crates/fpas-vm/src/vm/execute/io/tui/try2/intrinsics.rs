//! Try-2 VM intrinsic handlers (`Dialog.NewModal`, `Application.ExecView`, …).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::chrome::{
    try2_menu_bar_new, try2_menu_bar_set_menus, try2_status_line_new, try2_status_line_set_items,
};
use super::headless::try2_ensure_headless_app;
use super::modals::try2_exec_view;
use super::records::{
    TUI_BUTTON_TYPE, TUI_CHECK_BOX_TYPE, TUI_DIALOG_TYPE, TUI_INPUT_LINE_TYPE, TUI_LIST_BOX_TYPE,
    TUI_MEMO_TYPE, TUI_RADIO_BUTTON_TYPE, TUI_STATIC_TEXT_TYPE, TUI_TEXT_VIEWER_TYPE,
    TUI_WINDOW_TYPE,
};
use super::registry::ViewKind;
use super::views::{
    try2_button_new, try2_button_set_text, try2_check_box_checked, try2_check_box_new,
    try2_check_box_set_checked, try2_desktop_add, try2_dialog_add_button, try2_dialog_attach_child,
    try2_dialog_new_modal, try2_dialog_set_title, try2_input_line_new, try2_input_line_set_text,
    try2_input_line_text, try2_list_box_new, try2_list_box_selection, try2_list_box_set_items,
    try2_memo_new, try2_memo_set_text, try2_radio_button_new, try2_radio_button_selected,
    try2_radio_button_set_selected, try2_static_text_new, try2_static_text_set_text,
    try2_text_viewer_new, try2_text_viewer_set_text, try2_window_attach_child, try2_window_new,
    try2_window_set_title,
};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::handle_records::{
    HANDLE_FIELD, TUI_MENU_BAR_TYPE, TUI_STATUS_LINE_TYPE,
};
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
            TuiIntrinsic::ButtonNew => {
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
                let handle = try2_button_new(self, bounds, text, command, is_default, line)?;
                self.push(Self::turbo_vision_button_record(handle))?;
            }
            TuiIntrinsic::DialogAdd => {
                let (child_handle, child_kind) = self.pop_try2_child_handle(line)?;
                let dialog_handle = self.pop_try2_handle(TUI_DIALOG_TYPE, "Dialog", line)?;
                try2_dialog_attach_child(self, dialog_handle, child_handle, child_kind, line)?;
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
            TuiIntrinsic::Try2InjectCommand => {
                let command = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let command = u16::try_from(command).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("Command id {command} is outside the Turbo Vision u16 range"),
                        "Use a command id from 0 to 65535.",
                        line,
                    )
                })?;
                self.try2_inject_command(command, line)?;
            }
            TuiIntrinsic::WindowNew => {
                let title = self.pop_turbo_vision_string("Window title", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_window_new(self, bounds, title, line)?;
                self.push(Self::turbo_vision_window_record(handle))?;
            }
            TuiIntrinsic::WindowAdd => {
                let (child_handle, child_kind) = self.pop_try2_child_handle(line)?;
                let window_handle = self.pop_try2_handle(TUI_WINDOW_TYPE, "Window", line)?;
                try2_window_attach_child(self, window_handle, child_handle, child_kind, line)?;
            }
            TuiIntrinsic::DesktopAdd => {
                let window_handle = self.pop_try2_handle(TUI_WINDOW_TYPE, "Window", line)?;
                self.pop_tui_application(line)?;
                try2_desktop_add(self, window_handle, line)?;
            }
            TuiIntrinsic::StaticTextNew => {
                let text = self.pop_turbo_vision_string("StaticText text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_static_text_new(self, bounds, text, line)?;
                self.push(Self::turbo_vision_static_text_record(handle))?;
            }
            TuiIntrinsic::MenuBarNew => {
                let menus = self.parse_turbo_vision_menus(line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_menu_bar_new(self, bounds, menus, line)?;
                self.push(Self::turbo_vision_menu_bar_record(handle))?;
            }
            TuiIntrinsic::StatusLineNew => {
                let items = self.parse_turbo_vision_status_items(line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_status_line_new(self, bounds, items, line)?;
                self.push(Self::turbo_vision_status_line_record(handle))?;
            }
            TuiIntrinsic::CheckBoxNew => {
                let checked = self.pop_bool(line)?;
                let text = self.pop_turbo_vision_string("CheckBox text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_check_box_new(self, bounds, text, checked, line)?;
                self.push(Self::turbo_vision_check_box_record(handle))?;
            }
            TuiIntrinsic::InputLineNew => {
                let max_length = self.pop_int(line)?;
                let max_length = usize::try_from(max_length).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "InputLine MaxLength must be non-negative",
                        "Use a MaxLength value from 0 to the platform usize maximum.",
                        line,
                    )
                })?;
                let text = self.pop_turbo_vision_string("InputLine text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_input_line_new(self, bounds, text, max_length, line)?;
                self.push(Self::turbo_vision_input_line_record(handle))?;
            }
            TuiIntrinsic::CheckBoxChecked => {
                let handle = self.pop_try2_handle(TUI_CHECK_BOX_TYPE, "CheckBox", line)?;
                let checked = try2_check_box_checked(self, handle, line)?;
                self.push(Value::Boolean(checked))?;
            }
            TuiIntrinsic::CheckBoxSetChecked => {
                let checked = self.pop_bool(line)?;
                let handle = self.pop_try2_handle(TUI_CHECK_BOX_TYPE, "CheckBox", line)?;
                try2_check_box_set_checked(self, handle, checked, line)?;
            }
            TuiIntrinsic::InputLineText => {
                let handle = self.pop_try2_handle(TUI_INPUT_LINE_TYPE, "InputLine", line)?;
                let text = try2_input_line_text(self, handle, line)?;
                self.push(Value::Str(text))?;
            }
            TuiIntrinsic::InputLineSetText => {
                let text = self.pop_turbo_vision_string("InputLine text", line)?;
                let handle = self.pop_try2_handle(TUI_INPUT_LINE_TYPE, "InputLine", line)?;
                try2_input_line_set_text(self, handle, text, line)?;
            }
            TuiIntrinsic::ListBoxNew => {
                let command = self.pop_int(line)?;
                let command = u16::try_from(command).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "ListBox command id is outside the Turbo Vision u16 range",
                        "Use a command id from 0 to 65535.",
                        line,
                    )
                })?;
                let items = self.pop_turbo_vision_string_array("ListBox items", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_list_box_new(self, bounds, items, command, line)?;
                self.push(Self::turbo_vision_list_box_record(handle))?;
            }
            TuiIntrinsic::ListBoxSelection => {
                let handle = self.pop_try2_handle(TUI_LIST_BOX_TYPE, "ListBox", line)?;
                let selection = try2_list_box_selection(self, handle, line)?;
                self.push(Value::Integer(selection))?;
            }
            TuiIntrinsic::ListBoxSetItems => {
                let items = self.pop_turbo_vision_string_array("ListBox items", line)?;
                let handle = self.pop_try2_handle(TUI_LIST_BOX_TYPE, "ListBox", line)?;
                try2_list_box_set_items(self, handle, items, line)?;
            }
            TuiIntrinsic::RadioButtonNew => {
                let selected = self.pop_bool(line)?;
                let group_id = self.pop_int(line)?;
                let group_id = u16::try_from(group_id).map_err(|_| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "RadioButton group id is outside the Turbo Vision u16 range",
                        "Use a group id from 0 to 65535.",
                        line,
                    )
                })?;
                let text = self.pop_turbo_vision_string("RadioButton text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_radio_button_new(self, bounds, text, group_id, selected, line)?;
                self.push(Self::turbo_vision_radio_button_record(handle))?;
            }
            TuiIntrinsic::RadioButtonSelected => {
                let handle = self.pop_try2_handle(TUI_RADIO_BUTTON_TYPE, "RadioButton", line)?;
                let selected = try2_radio_button_selected(self, handle, line)?;
                self.push(Value::Boolean(selected))?;
            }
            TuiIntrinsic::RadioButtonSetSelected => {
                let selected = self.pop_bool(line)?;
                let handle = self.pop_try2_handle(TUI_RADIO_BUTTON_TYPE, "RadioButton", line)?;
                try2_radio_button_set_selected(self, handle, selected, line)?;
            }
            TuiIntrinsic::MemoNew => {
                let text = self.pop_turbo_vision_string("Memo text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_memo_new(self, bounds, text, line)?;
                self.push(Self::turbo_vision_memo_record(handle))?;
            }
            TuiIntrinsic::MemoSetText => {
                let text = self.pop_turbo_vision_string("Memo text", line)?;
                let handle = self.pop_try2_handle(TUI_MEMO_TYPE, "Memo", line)?;
                try2_memo_set_text(self, handle, text, line)?;
            }
            TuiIntrinsic::TextViewerNew => {
                let text = self.pop_turbo_vision_string("TextViewer text", line)?;
                let bounds = self.pop_turbo_vision_rect(line)?;
                let handle = try2_text_viewer_new(self, bounds, text, line)?;
                self.push(Self::turbo_vision_text_viewer_record(handle))?;
            }
            TuiIntrinsic::TextViewerSetText => {
                let text = self.pop_turbo_vision_string("TextViewer text", line)?;
                let handle = self.pop_try2_handle(TUI_TEXT_VIEWER_TYPE, "TextViewer", line)?;
                try2_text_viewer_set_text(self, handle, text, line)?;
            }
            TuiIntrinsic::StaticTextSetText => {
                let text = self.pop_turbo_vision_string("StaticText text", line)?;
                let handle = self.pop_try2_handle(TUI_STATIC_TEXT_TYPE, "StaticText", line)?;
                try2_static_text_set_text(self, handle, text, line)?;
            }
            TuiIntrinsic::ButtonSetText => {
                let text = self.pop_turbo_vision_string("Button text", line)?;
                let handle = self.pop_try2_handle(TUI_BUTTON_TYPE, "Button", line)?;
                try2_button_set_text(self, handle, text, line)?;
            }
            TuiIntrinsic::DialogSetTitle => {
                let title = self.pop_turbo_vision_string("Dialog title", line)?;
                let handle = self.pop_try2_handle(TUI_DIALOG_TYPE, "Dialog", line)?;
                try2_dialog_set_title(self, handle, title, line)?;
            }
            TuiIntrinsic::WindowSetTitle => {
                let title = self.pop_turbo_vision_string("Window title", line)?;
                let handle = self.pop_try2_handle(TUI_WINDOW_TYPE, "Window", line)?;
                try2_window_set_title(self, handle, title, line)?;
            }
            TuiIntrinsic::MenuBarSetMenus => {
                let menus = self.parse_turbo_vision_menus(line)?;
                let handle = self.pop_try2_handle(TUI_MENU_BAR_TYPE, "MenuBar", line)?;
                try2_menu_bar_set_menus(self, handle, menus, line)?;
            }
            TuiIntrinsic::StatusLineSetItems => {
                let items = self.parse_turbo_vision_status_items(line)?;
                let handle = self.pop_try2_handle(TUI_STATUS_LINE_TYPE, "StatusLine", line)?;
                try2_status_line_set_items(self, handle, items, line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn pop_try2_child_handle(&mut self, line: SourceLocation) -> Result<(u32, ViewKind), VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_BUTTON_TYPE => Ok((
                self.decode_try2_handle(&fields, "Button", line)?,
                ViewKind::Button,
            )),
            Value::Record { type_name, fields } if type_name == TUI_STATIC_TEXT_TYPE => Ok((
                self.decode_try2_handle(&fields, "StaticText", line)?,
                ViewKind::StaticText,
            )),
            Value::Record { type_name, fields } if type_name == TUI_CHECK_BOX_TYPE => Ok((
                self.decode_try2_handle(&fields, "CheckBox", line)?,
                ViewKind::CheckBox,
            )),
            Value::Record { type_name, fields } if type_name == TUI_INPUT_LINE_TYPE => Ok((
                self.decode_try2_handle(&fields, "InputLine", line)?,
                ViewKind::InputLine,
            )),
            Value::Record { type_name, fields } if type_name == TUI_LIST_BOX_TYPE => Ok((
                self.decode_try2_handle(&fields, "ListBox", line)?,
                ViewKind::ListBox,
            )),
            Value::Record { type_name, fields } if type_name == TUI_RADIO_BUTTON_TYPE => Ok((
                self.decode_try2_handle(&fields, "RadioButton", line)?,
                ViewKind::RadioButton,
            )),
            Value::Record { type_name, fields } if type_name == TUI_MEMO_TYPE => Ok((
                self.decode_try2_handle(&fields, "Memo", line)?,
                ViewKind::Memo,
            )),
            Value::Record { type_name, fields } if type_name == TUI_TEXT_VIEWER_TYPE => Ok((
                self.decode_try2_handle(&fields, "TextViewer", line)?,
                ViewKind::TextViewer,
            )),
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Expected a try-2 child widget handle, got {}",
                    other.type_name()
                ),
                "Pass a child handle from `Button.New`, `StaticText.New`, `CheckBox.New`, `InputLine.New`, `ListBox.New`, `RadioButton.New`, `Memo.New`, or `TextViewer.New`.",
                line,
            )),
        }
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

    fn try2_inject_keyboard(&mut self, key_code: u16, line: SourceLocation) -> Result<(), VmError> {
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

    fn try2_inject_command(&mut self, command: u16, line: SourceLocation) -> Result<(), VmError> {
        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Try2InjectCommand is only supported in headless OpenForTest sessions",
                "Call `Application.OpenForTest` before injecting synthetic commands.",
                line,
            ));
        }
        try2_ensure_headless_app(self, line)?;
        if let Some(app) = self.headless_tv_app.as_ref() {
            app.push_command(command);
        }
        Ok(())
    }
}
