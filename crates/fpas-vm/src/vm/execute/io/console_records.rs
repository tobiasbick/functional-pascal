//! Runtime conversion between console host events and FPAS records.

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::ConsoleKeyEvent;

impl Worker {
    pub(in crate::vm::execute::io) fn key_event_record(event: fpas_std::ConsoleKeyEvent) -> Value {
        Value::Record {
            type_name: "Std.Console.KeyEvent".into(),
            fields: vec![
                ("kind".into(), Value::Integer(event.kind as i64)),
                ("ch".into(), key_event_char_value(event.ch)),
                ("shift".into(), Value::Boolean(event.shift)),
                ("ctrl".into(), Value::Boolean(event.ctrl)),
                ("alt".into(), Value::Boolean(event.alt)),
                ("meta".into(), Value::Boolean(event.meta)),
            ],
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Console event records have a fixed field layout mirroring Std.Console.Event"
    )]
    fn console_event_record_fields(
        kind: usize,
        key: ConsoleKeyEvent,
        mouse_action: usize,
        mouse_button: usize,
        mouse_x: i64,
        mouse_y: i64,
        width: i64,
        height: i64,
        text: String,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    ) -> Value {
        Value::Record {
            type_name: "Std.Console.Event".into(),
            fields: vec![
                ("kind".into(), Value::Integer(kind as i64)),
                ("key".into(), Self::key_event_record(key)),
                ("mouse_action".into(), Value::Integer(mouse_action as i64)),
                ("mouse_button".into(), Value::Integer(mouse_button as i64)),
                ("mouse_x".into(), Value::Integer(mouse_x)),
                ("mouse_y".into(), Value::Integer(mouse_y)),
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
                ("text".into(), Value::Str(text)),
                ("shift".into(), Value::Boolean(shift)),
                ("ctrl".into(), Value::Boolean(ctrl)),
                ("alt".into(), Value::Boolean(alt)),
                ("meta".into(), Value::Boolean(meta)),
            ],
        }
    }

    /// Builds one `Std.Console.Event` record from the runtime console event model.
    pub(in crate::vm::execute::io) fn console_event_record(event: fpas_std::ConsoleEvent) -> Value {
        let fpas_std::ConsoleEvent {
            kind,
            key,
            mouse_action,
            mouse_button,
            mouse_x,
            mouse_y,
            width,
            height,
            text,
            shift,
            ctrl,
            alt,
            meta,
        } = event;
        Self::console_event_record_fields(
            kind,
            key,
            mouse_action,
            mouse_button,
            mouse_x,
            mouse_y,
            width,
            height,
            text,
            shift,
            ctrl,
            alt,
            meta,
        )
    }

    /// Pop a `Std.Console.KeyEvent` record from the stack.
    pub(in crate::vm::execute::io) fn pop_console_key_event(
        &mut self,
        line: SourceLocation,
    ) -> Result<ConsoleKeyEvent, VmError> {
        const KEY: &str = "Std.Console.KeyEvent";
        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == KEY => {
                Self::console_key_event_from_fields(&fields, line)
            }
            Value::Record { type_name, fields } if type_name == "<record>" => {
                Self::console_key_event_from_fields(&fields, line)
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {KEY}, got {}", other.type_name()),
                "Pass a `Std.Console.KeyEvent` value.",
                line,
            )),
        }
    }

    fn console_key_event_from_fields(
        fields: &[(String, Value)],
        line: SourceLocation,
    ) -> Result<ConsoleKeyEvent, VmError> {
        let field = |name: &str| -> Result<&Value, VmError> {
            fields
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v)
                .ok_or_else(|| {
                    internal_error(
                        format!("Std.Console.KeyEvent missing field `{name}`"),
                        "This indicates a compiler/runtime mismatch.",
                        line,
                    )
                })
        };

        let kind = match field("kind")? {
            Value::Integer(i) if *i >= 0 => *i as usize,
            _ => {
                return Err(internal_error(
                    "Std.Console.KeyEvent.kind must be a non-negative integer",
                    "This indicates a compiler/runtime mismatch.",
                    line,
                ));
            }
        };
        let ch = match field("ch")? {
            Value::Str(s) => key_event_char_from_string(s),
            _ => {
                return Err(internal_error(
                    "Std.Console.KeyEvent.ch must be a string",
                    "This indicates a compiler/runtime mismatch.",
                    line,
                ));
            }
        };
        let read_bool = |name: &str| -> Result<bool, VmError> {
            match field(name)? {
                Value::Boolean(b) => Ok(*b),
                _ => Err(internal_error(
                    format!("Std.Console.KeyEvent.{name} must be a boolean"),
                    "This indicates a compiler/runtime mismatch.",
                    line,
                )),
            }
        };
        let shift = read_bool("shift")?;
        let ctrl = read_bool("ctrl")?;
        let alt = read_bool("alt")?;
        let meta = read_bool("meta")?;

        Ok(ConsoleKeyEvent::new(kind, ch, shift, ctrl, alt, meta))
    }
}

/// Maps a host key character to the FPAS `KeyEvent.ch` string field.
fn key_event_char_value(ch: char) -> Value {
    if ch == '\0' {
        Value::Str(String::new())
    } else {
        Value::Str(ch.to_string())
    }
}

/// Reads the host key character from a FPAS `KeyEvent.ch` string field.
fn key_event_char_from_string(s: &str) -> char {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (None, _) => '\0',
        (Some(c), None) => c,
        (Some(c), _) => c,
    }
}
