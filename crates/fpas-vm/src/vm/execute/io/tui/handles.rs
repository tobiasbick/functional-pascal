//! FPAS handle records and value decoding for Turbo Vision objects.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

const TUI_DIALOG_TYPE: &str = "Std.Tui.Dialog";
const TUI_WINDOW_TYPE: &str = "Std.Tui.Window";
const TUI_BUTTON_TYPE: &str = "Std.Tui.Button";
const TUI_STATIC_TEXT_TYPE: &str = "Std.Tui.StaticText";
const TUI_MEMO_TYPE: &str = "Std.Tui.Memo";
const TUI_TEXT_VIEWER_TYPE: &str = "Std.Tui.TextViewer";
const TUI_INPUT_LINE_TYPE: &str = "Std.Tui.InputLine";
const TUI_DIALOG_RESULT_TYPE: &str = "Std.Tui.DialogResult";
const TUI_LIST_BOX_TYPE: &str = "Std.Tui.ListBox";
const TUI_CHECK_BOX_TYPE: &str = "Std.Tui.CheckBox";
const TUI_RADIO_BUTTON_TYPE: &str = "Std.Tui.RadioButton";
const TUI_MENU_BAR_TYPE: &str = "Std.Tui.MenuBar";
const TUI_STATUS_LINE_TYPE: &str = "Std.Tui.StatusLine";
const HANDLE_FIELD: &str = "__id";
const TUI_RECT_TYPE: &str = "Std.Tui.Rect";

impl Worker {
    pub(super) fn turbo_vision_dialog_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_DIALOG_TYPE, handle)
    }

    pub(super) fn turbo_vision_window_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_WINDOW_TYPE, handle)
    }

    pub(super) fn turbo_vision_button_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_BUTTON_TYPE, handle)
    }

    pub(super) fn turbo_vision_static_text_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_STATIC_TEXT_TYPE, handle)
    }

    pub(super) fn turbo_vision_memo_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_MEMO_TYPE, handle)
    }

    pub(super) fn turbo_vision_text_viewer_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_TEXT_VIEWER_TYPE, handle)
    }

    pub(super) fn turbo_vision_input_line_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_INPUT_LINE_TYPE, handle)
    }

    pub(super) fn turbo_vision_list_box_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_LIST_BOX_TYPE, handle)
    }

    pub(super) fn turbo_vision_check_box_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_CHECK_BOX_TYPE, handle)
    }

    pub(super) fn turbo_vision_radio_button_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_RADIO_BUTTON_TYPE, handle)
    }

    pub(super) fn turbo_vision_menu_bar_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_MENU_BAR_TYPE, handle)
    }

    pub(super) fn turbo_vision_status_line_record(handle: u32) -> Value {
        turbo_vision_handle_record(TUI_STATUS_LINE_TYPE, handle)
    }

    pub(super) fn pop_turbo_vision_dialog_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_DIALOG_TYPE, "Dialog", line)
    }

    pub(super) fn pop_turbo_vision_input_line_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_INPUT_LINE_TYPE, "InputLine", line)
    }

    pub(super) fn pop_turbo_vision_check_box_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_CHECK_BOX_TYPE, "CheckBox", line)
    }

    pub(super) fn pop_turbo_vision_radio_button_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_RADIO_BUTTON_TYPE, "RadioButton", line)
    }

    pub(super) fn push_dialog_result(&mut self, command: i64) -> Result<(), VmError> {
        self.push(Value::Record {
            type_name: TUI_DIALOG_RESULT_TYPE.into(),
            fields: vec![("command".into(), Value::Integer(command))],
        })
    }

    pub(super) fn pop_turbo_vision_window_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_WINDOW_TYPE, "Window", line)
    }

    pub(super) fn pop_turbo_vision_parent_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<TurboVisionParentHandle, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_DIALOG_TYPE => {
                Ok(TurboVisionParentHandle::Dialog(
                    self.decode_turbo_vision_handle_record(&fields, "Dialog", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_WINDOW_TYPE => {
                Ok(TurboVisionParentHandle::Window(
                    self.decode_turbo_vision_handle_record(&fields, "Window", line)?,
                ))
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Parent handle expected {TUI_DIALOG_TYPE} or {TUI_WINDOW_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a handle from `Application.CreateDialog` or `Application.CreateWindow`.",
                line,
            )),
        }
    }

    pub(super) fn pop_turbo_vision_list_box_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_LIST_BOX_TYPE, "ListBox", line)
    }

    pub(super) fn pop_turbo_vision_checked_control_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<CheckedControlHandle, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_CHECK_BOX_TYPE => {
                Ok(CheckedControlHandle::CheckBox(
                    self.decode_turbo_vision_handle_record(&fields, "CheckBox", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_RADIO_BUTTON_TYPE => {
                Ok(CheckedControlHandle::RadioButton(
                    self.decode_turbo_vision_handle_record(&fields, "RadioButton", line)?,
                ))
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Checked control expected {TUI_CHECK_BOX_TYPE} or {TUI_RADIO_BUTTON_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a handle from `Application.CreateCheckBox` or `Application.CreateRadioButton`.",
                line,
            )),
        }
    }

    pub(super) fn pop_turbo_vision_button_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_BUTTON_TYPE, "Button", line)
    }

    pub(super) fn pop_turbo_vision_menu_bar_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_MENU_BAR_TYPE, "MenuBar", line)
    }

    pub(super) fn pop_turbo_vision_status_line_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        self.pop_turbo_vision_handle(TUI_STATUS_LINE_TYPE, "StatusLine", line)
    }

    pub(super) fn pop_turbo_vision_child_handle(
        &mut self,
        line: SourceLocation,
    ) -> Result<TurboVisionChildHandle, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == TUI_BUTTON_TYPE => {
                Ok(TurboVisionChildHandle::Button(
                    self.decode_turbo_vision_handle_record(&fields, "Button", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_STATIC_TEXT_TYPE => {
                Ok(TurboVisionChildHandle::StaticText(
                    self.decode_turbo_vision_handle_record(&fields, "StaticText", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_MEMO_TYPE => {
                Ok(TurboVisionChildHandle::Memo(
                    self.decode_turbo_vision_handle_record(&fields, "Memo", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_TEXT_VIEWER_TYPE => {
                Ok(TurboVisionChildHandle::TextViewer(
                    self.decode_turbo_vision_handle_record(&fields, "TextViewer", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_INPUT_LINE_TYPE => {
                Ok(TurboVisionChildHandle::InputLine(
                    self.decode_turbo_vision_handle_record(&fields, "InputLine", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_LIST_BOX_TYPE => {
                Ok(TurboVisionChildHandle::ListBox(
                    self.decode_turbo_vision_handle_record(&fields, "ListBox", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_CHECK_BOX_TYPE => {
                Ok(TurboVisionChildHandle::CheckBox(
                    self.decode_turbo_vision_handle_record(&fields, "CheckBox", line)?,
                ))
            }
            Value::Record { type_name, fields } if type_name == TUI_RADIO_BUTTON_TYPE => {
                Ok(TurboVisionChildHandle::RadioButton(
                    self.decode_turbo_vision_handle_record(&fields, "RadioButton", line)?,
                ))
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Child handle expected {TUI_BUTTON_TYPE}, {TUI_STATIC_TEXT_TYPE}, {TUI_MEMO_TYPE}, {TUI_TEXT_VIEWER_TYPE}, {TUI_INPUT_LINE_TYPE}, {TUI_LIST_BOX_TYPE}, {TUI_CHECK_BOX_TYPE}, or {TUI_RADIO_BUTTON_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a handle from `Application.CreateButton`, `Application.CreateStaticText`, `Application.CreateMemo`, `Application.CreateTextViewer`, `Application.CreateInputLine`, `Application.CreateListBox`, `Application.CreateCheckBox`, or `Application.CreateRadioButton`.",
                line,
            )),
        }
    }

    pub(super) fn pop_turbo_vision_rect(&mut self, line: SourceLocation) -> Result<Rect, VmError> {
        let value = self.pop(line)?;
        let Value::Record { type_name, fields } = value else {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {TUI_RECT_TYPE}, got {}", value.type_name()),
                "Pass a Std.Tui.Rect value.",
                line,
            ));
        };
        if type_name != TUI_RECT_TYPE {
            return Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {TUI_RECT_TYPE}, got {type_name}"),
                "Pass a Std.Tui.Rect value.",
                line,
            ));
        }

        let x = self.turbo_vision_i16_field(&fields, "x", line)?;
        let y = self.turbo_vision_i16_field(&fields, "y", line)?;
        let width = self.turbo_vision_positive_i16_field(&fields, "width", line)?;
        let height = self.turbo_vision_positive_i16_field(&fields, "height", line)?;
        Ok(Rect::from_coords(x, y, width, height))
    }

    pub(super) fn pop_turbo_vision_string(
        &mut self,
        label: &'static str,
        line: SourceLocation,
    ) -> Result<String, VmError> {
        match self.pop(line)? {
            Value::Str(text) => Ok(text),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("{label} must be string, got {}", other.type_name()),
                "Pass a string value.",
                line,
            )),
        }
    }

    pub(super) fn pop_optional_string(
        &mut self,
        label: &'static str,
        line: SourceLocation,
    ) -> Result<Option<String>, VmError> {
        match self.pop(line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => match *inner {
                Value::Str(text) => Ok(Some(text)),
                other => Err(runtime_error(
                    TYPE_MISMATCH_CODE,
                    format!(
                        "{label} must be Option of string, got Some({})",
                        other.type_name()
                    ),
                    "Pass `None` or `Some('path')`.",
                    line,
                )),
            },
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "{label} must be Option of string, got {}",
                    other.type_name()
                ),
                "Pass `None` or `Some('path')`.",
                line,
            )),
        }
    }

    pub(super) fn push_optional_string(&mut self, value: Option<String>) -> Result<(), VmError> {
        match value {
            Some(text) => self.push(Value::OptionSome(Box::new(Value::Str(text))))?,
            None => self.push(Value::OptionNone)?,
        }
        Ok(())
    }

    fn pop_turbo_vision_handle(
        &mut self,
        expected_type: &'static str,
        label: &'static str,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == expected_type => {
                self.decode_turbo_vision_handle_record(&fields, label, line)
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "{label} handle expected {expected_type}, got {}",
                    other.type_name()
                ),
                "Pass a handle returned by the matching Std.Tui.Application constructor.",
                line,
            )),
        }
    }

    fn decode_turbo_vision_handle_record(
        &self,
        fields: &[(String, Value)],
        label: &'static str,
        line: SourceLocation,
    ) -> Result<u32, VmError> {
        let Some(Value::Integer(raw)) = fields
            .iter()
            .find(|(name, _)| name == HANDLE_FIELD)
            .map(|(_, value)| value)
        else {
            return Err(turbo_vision_handle_error(
                label,
                "missing handle token",
                line,
            ));
        };
        if !(1..=i64::from(u32::MAX)).contains(raw) {
            return Err(turbo_vision_handle_error(
                label,
                "handle token is out of range",
                line,
            ));
        }
        Ok(*raw as u32)
    }

    fn turbo_vision_i16_field(
        &self,
        fields: &[(String, Value)],
        field_name: &'static str,
        line: SourceLocation,
    ) -> Result<i16, VmError> {
        let value = self.integer_record_field(fields, field_name, line)?;
        i16::try_from(value).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Rect.{field_name}={value} is outside the Turbo Vision i16 range"),
                "Use coordinates in the range -32768..32767.",
                line,
            )
        })
    }

    fn turbo_vision_positive_i16_field(
        &self,
        fields: &[(String, Value)],
        field_name: &'static str,
        line: SourceLocation,
    ) -> Result<i16, VmError> {
        let value = self.turbo_vision_i16_field(fields, field_name, line)?;
        if value <= 0 {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Rect.{field_name} must be positive, got {value}"),
                "Use a positive width and height for Turbo Vision controls.",
                line,
            ));
        }
        Ok(value)
    }
}

fn turbo_vision_handle_record(type_name: &'static str, handle: u32) -> Value {
    Value::Record {
        type_name: type_name.into(),
        fields: vec![(HANDLE_FIELD.into(), Value::Integer(i64::from(handle)))],
    }
}

pub(super) enum TurboVisionParentHandle {
    Dialog(u32),
    Window(u32),
}

pub(super) enum TurboVisionChildHandle {
    Button(u32),
    StaticText(u32),
    Memo(u32),
    TextViewer(u32),
    InputLine(u32),
    ListBox(u32),
    CheckBox(u32),
    RadioButton(u32),
}

pub(super) enum CheckedControlHandle {
    CheckBox(u32),
    RadioButton(u32),
}

fn turbo_vision_handle_error(
    label: &'static str,
    detail: &'static str,
    line: SourceLocation,
) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("{label} handle is invalid: {detail}"),
        "Use handles returned by the Turbo Vision Std.Tui constructors in the same application session.",
        line,
    )
}
