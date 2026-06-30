//! `Std.Tui` value constructors: `Application`, `Size`, and `ScreenCell` records.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use fpas_bytecode::Value;
use fpas_std::ViewRect;

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_SCREEN_CELL_TYPE: &str = "Std.Tui.ScreenCell";

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

    /// Constructs a `Std.Tui.ScreenCell` record with `ch`, `fg`, and `bg` fields.
    pub(in crate::vm::execute::io) fn tui_screen_cell_record(ch: char, fg: u8, bg: u8) -> Value {
        Value::Record {
            type_name: TUI_SCREEN_CELL_TYPE.into(),
            fields: vec![
                ("ch".into(), Value::Str(ch.to_string())),
                ("fg".into(), Value::Integer(i64::from(fg))),
                ("bg".into(), Value::Integer(i64::from(bg))),
            ],
        }
    }

    /// Constructs a `Std.Tui.Rect` record with `x`, `y`, `width`, and `height` fields.
    pub(in crate::vm::execute::io) fn tui_rect_record(rect: ViewRect) -> Value {
        Value::Record {
            type_name: "Std.Tui.Rect".into(),
            fields: vec![
                ("x".into(), Value::Integer(rect.x)),
                ("y".into(), Value::Integer(rect.y)),
                ("width".into(), Value::Integer(rect.width)),
                ("height".into(), Value::Integer(rect.height)),
            ],
        }
    }
}
