//! `Std.Tui` value constructors: `Application`, `Size`, and `Rect` records.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use crate::vm::Worker;
use fpas_bytecode::Value;
use fpas_std::ViewRect;

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_RECT_TYPE: &str = "Std.Tui.Rect";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";

impl Worker {
    /// Constructs an empty `Std.Tui.Application` record.
    pub(in crate::vm::execute::io) fn tui_application_record() -> Value {
        Value::Record {
            type_name: TUI_APPLICATION_TYPE.into(),
            fields: vec![],
        }
    }

    /// Constructs a `Std.Tui.Size` record with `width` and `height` fields.
    pub(in crate::vm::execute::io) fn tui_size_record(width: i64, height: i64) -> Value {
        Value::Record {
            type_name: TUI_SIZE_TYPE.into(),
            fields: vec![
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
            ],
        }
    }

    /// Constructs a `Std.Tui.Rect` record with `x`, `y`, `width`, and `height` fields.
    pub(in crate::vm::execute::io) fn tui_rect_record(rect: ViewRect) -> Value {
        Value::Record {
            type_name: TUI_RECT_TYPE.into(),
            fields: vec![
                ("x".into(), Value::Integer(rect.x)),
                ("y".into(), Value::Integer(rect.y)),
                ("width".into(), Value::Integer(rect.width)),
                ("height".into(), Value::Integer(rect.height)),
            ],
        }
    }
}
