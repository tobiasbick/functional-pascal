//! `Std.Tui` value constructors: `Application`, `Size`, and `TuiEvent` records.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use crate::vm::Worker;
use fpas_bytecode::Value;
use fpas_std::{ConsoleKeyEvent, TuiEvent, UiEvent, ViewRect};

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_RECT_TYPE: &str = "Std.Tui.Rect";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_EVENT_TYPE: &str = "Std.Tui.TuiEvent";

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

    /// Constructs a placeholder key event (kind = `Unknown`) used as a filler in non-key events.
    fn tui_unknown_key_event() -> Value {
        Self::key_event_record(ConsoleKeyEvent::new(
            fpas_std::key_event::key_kind_index("Unknown"),
            '\0',
            false,
            false,
            false,
            false,
        ))
    }

    /// Converts a [`TuiEvent`] into a `Std.Tui.TuiEvent` record.
    pub(in crate::vm::execute::io) fn tui_event_record(event: TuiEvent) -> Value {
        match event {
            TuiEvent::Key(key) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(0)),
                    ("key".into(), Self::key_event_record(key)),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            TuiEvent::Resize { width, height, .. } => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(1)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(width, height)),
                ],
            },
            TuiEvent::Mouse(_) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(2)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            // Paste/Focus are dispatch-only; kind integers 3/4/5 are beyond the declared
            // Std.Tui.EventKind variants but won't crash legacy poll-style callers.
            TuiEvent::Paste(_) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(3)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            TuiEvent::FocusGained => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(4)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            TuiEvent::FocusLost => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(5)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
        }
    }

    /// Converts a shared [`UiEvent`] into a `Std.Tui.TuiEvent` record when representable.
    pub(in crate::vm::execute::io) fn tui_ui_event_record(event: UiEvent) -> Option<Value> {
        match event {
            UiEvent::Resize(resize) => Some(Self::tui_event_record(TuiEvent::Resize {
                old_width: resize.old_width.unwrap_or(0),
                old_height: resize.old_height.unwrap_or(0),
                width: resize.width,
                height: resize.height,
            })),
            UiEvent::Key(key) => Some(Self::tui_event_record(TuiEvent::Key(key))),
            UiEvent::Mouse(mouse) => Some(Self::tui_event_record(TuiEvent::Mouse(mouse))),
            UiEvent::Paste(text) => Some(Self::tui_event_record(TuiEvent::Paste(text))),
            UiEvent::FocusGained => Some(Self::tui_event_record(TuiEvent::FocusGained)),
            UiEvent::FocusLost => Some(Self::tui_event_record(TuiEvent::FocusLost)),
            UiEvent::CloseRequested | UiEvent::Wheel(_) => None,
        }
    }
}
