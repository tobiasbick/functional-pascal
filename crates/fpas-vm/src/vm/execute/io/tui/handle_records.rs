//! FPAS handle record construction for Turbo Vision objects.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use fpas_bytecode::Value;

pub(in crate::vm::execute::io::tui) const TUI_DIALOG_TYPE: &str = "Std.Tui.Dialog";
pub(in crate::vm::execute::io::tui) const TUI_WINDOW_TYPE: &str = "Std.Tui.Window";
pub(in crate::vm::execute::io::tui) const TUI_BUTTON_TYPE: &str = "Std.Tui.Button";
pub(in crate::vm::execute::io::tui) const TUI_STATIC_TEXT_TYPE: &str = "Std.Tui.StaticText";
pub(in crate::vm::execute::io::tui) const TUI_MEMO_TYPE: &str = "Std.Tui.Memo";
pub(in crate::vm::execute::io::tui) const TUI_TEXT_VIEWER_TYPE: &str = "Std.Tui.TextViewer";
pub(in crate::vm::execute::io::tui) const TUI_INPUT_LINE_TYPE: &str = "Std.Tui.InputLine";
pub(in crate::vm::execute::io::tui) const TUI_DIALOG_RESULT_TYPE: &str = "Std.Tui.DialogResult";
pub(in crate::vm::execute::io::tui) const TUI_LIST_BOX_TYPE: &str = "Std.Tui.ListBox";
pub(in crate::vm::execute::io::tui) const TUI_CHECK_BOX_TYPE: &str = "Std.Tui.CheckBox";
pub(in crate::vm::execute::io::tui) const TUI_RADIO_BUTTON_TYPE: &str = "Std.Tui.RadioButton";
pub(in crate::vm::execute::io::tui) const TUI_MENU_BAR_TYPE: &str = "Std.Tui.MenuBar";
pub(in crate::vm::execute::io::tui) const TUI_STATUS_LINE_TYPE: &str = "Std.Tui.StatusLine";
pub(in crate::vm::execute::io::tui) const HANDLE_FIELD: &str = "__id";
pub(in crate::vm::execute::io::tui) const TUI_RECT_TYPE: &str = "Std.Tui.Rect";

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
}

fn turbo_vision_handle_record(type_name: &'static str, handle: u32) -> Value {
    Value::Record {
        type_name: type_name.into(),
        fields: vec![(HANDLE_FIELD.into(), Value::Integer(i64::from(handle)))],
    }
}
