//! `Std.Tui` value constructors: `Application`, `ViewId`, `Size`, and `Rect` records.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{ViewId, ViewLayout, ViewRect};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_VIEW_ID_TYPE: &str = "Std.Tui.ViewId";
const TUI_VIEW_ID_RAW_FIELD: &str = "__id";
const TUI_RECT_TYPE: &str = "Std.Tui.Rect";
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

    /// Constructs a `Std.Tui.ViewId` record backed by the host registry token.
    pub(in crate::vm::execute::io) fn tui_view_id_record(view_id: ViewId) -> Value {
        Value::Record {
            type_name: TUI_VIEW_ID_TYPE.into(),
            fields: vec![(
                TUI_VIEW_ID_RAW_FIELD.into(),
                Value::Integer(i64::from(view_id.raw())),
            )],
        }
    }

    /// Reads the host token from a `Std.Tui.ViewId` runtime value.
    pub(in crate::vm::execute::io) fn tui_view_id_from_value(
        value: &Value,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        match value {
            Value::Record { type_name, fields } if type_name == TUI_VIEW_ID_TYPE => {
                let Some(Value::Integer(raw)) = fields
                    .iter()
                    .find(|(name, _)| name == TUI_VIEW_ID_RAW_FIELD)
                    .map(|(_, value)| value)
                else {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Std.Tui.ViewId is missing its internal host token",
                        "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                        line,
                    ));
                };
                if *raw < 0 {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("ViewId host token {raw} is out of range"),
                        "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                        line,
                    ));
                }
                Ok(ViewId::from_raw(*raw as u32))
            }
            other => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Expected Std.Tui.ViewId, got {}", other.type_name()),
                "Pass a view handle returned by `Application.HostRegisterView` or a host widget constructor.",
                line,
            )),
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
            type_name: TUI_RECT_TYPE.into(),
            fields: vec![
                ("x".into(), Value::Integer(rect.x)),
                ("y".into(), Value::Integer(rect.y)),
                ("width".into(), Value::Integer(rect.width)),
                ("height".into(), Value::Integer(rect.height)),
            ],
        }
    }

    /// Decode a `Std.Tui.ViewLayout` record from the VM stack.
    pub(in crate::vm::execute::io) fn pop_tui_view_layout(
        &mut self,
        line: SourceLocation,
    ) -> Result<ViewLayout, VmError> {
        let value = self.pop(line)?;
        let Value::Record { fields, .. } = value else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Expected a `Std.Tui.ViewLayout` record.",
                "Pass a `ViewLayout` record with anchor and margin fields.",
                line,
            ));
        };
        Ok(ViewLayout {
            anchor_left: Self::bool_record_field(&fields, "anchorLeft", line)?,
            anchor_top: Self::bool_record_field(&fields, "anchorTop", line)?,
            anchor_right: Self::bool_record_field(&fields, "anchorRight", line)?,
            anchor_bottom: Self::bool_record_field(&fields, "anchorBottom", line)?,
            margin_left: self.integer_record_field(&fields, "marginLeft", line)?,
            margin_top: self.integer_record_field(&fields, "marginTop", line)?,
            margin_right: self.integer_record_field(&fields, "marginRight", line)?,
            margin_bottom: self.integer_record_field(&fields, "marginBottom", line)?,
        })
    }
}
