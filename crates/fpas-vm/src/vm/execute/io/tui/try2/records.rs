//! Try-2 FPAS handle records (`__id` field).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use fpas_bytecode::Value;

pub(in crate::vm::execute::io::tui::try2) const HANDLE_FIELD: &str = "__id";

pub(in crate::vm::execute::io::tui::try2) fn handle_record(
    type_name: &'static str,
    handle: u32,
) -> Value {
    Value::Record {
        type_name: type_name.into(),
        fields: vec![(HANDLE_FIELD.into(), Value::Integer(i64::from(handle)))],
    }
}

pub(in crate::vm::execute::io::tui::try2) const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
pub(in crate::vm::execute::io::tui::try2) const TUI_DIALOG_TYPE: &str = "Std.Tui.Dialog";
pub(in crate::vm::execute::io::tui::try2) const TUI_WINDOW_TYPE: &str = "Std.Tui.Window";
pub(in crate::vm::execute::io::tui::try2) const TUI_STATIC_TEXT_TYPE: &str = "Std.Tui.StaticText";
pub(in crate::vm::execute::io::tui::try2) const TUI_BUTTON_TYPE: &str = "Std.Tui.Button";
