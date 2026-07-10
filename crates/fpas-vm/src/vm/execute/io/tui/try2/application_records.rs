//! Try-2 `Std.Tui` value constructors: `Application` and `Size` records.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use fpas_bytecode::Value;

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
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
}
