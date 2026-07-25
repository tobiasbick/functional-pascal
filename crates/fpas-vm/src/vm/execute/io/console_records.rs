//! Runtime conversion between console host events and FPAS records.

use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::ConsoleKeyEvent;

impl Worker {
    pub(in crate::vm::execute::io) fn key_event_record(event: ConsoleKeyEvent) -> Value {
        Value::record(
            "Std.Console.KeyEvent".into(),
            vec![
                ("kind".into(), Value::Integer(event.kind as i64)),
                ("ch".into(), key_event_char_value(event.ch)),
                ("shift".into(), Value::Boolean(event.shift)),
                ("ctrl".into(), Value::Boolean(event.ctrl)),
                ("alt".into(), Value::Boolean(event.alt)),
                ("meta".into(), Value::Boolean(event.meta)),
            ],
        )
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
        Value::record(
            "Std.Console.Event".into(),
            vec![
                ("kind".into(), Value::Integer(kind as i64)),
                ("key".into(), Self::key_event_record(key)),
                ("mouse_action".into(), Value::Integer(mouse_action as i64)),
                ("mouse_button".into(), Value::Integer(mouse_button as i64)),
                ("mouse_x".into(), Value::Integer(mouse_x)),
                ("mouse_y".into(), Value::Integer(mouse_y)),
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
                ("text".into(), Value::Str(text.into())),
                ("shift".into(), Value::Boolean(shift)),
                ("ctrl".into(), Value::Boolean(ctrl)),
                ("alt".into(), Value::Boolean(alt)),
                ("meta".into(), Value::Boolean(meta)),
            ],
        )
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

    /// Pops a `Std.Console.KeyEvent` record from the stack.
    pub(in crate::vm::execute::io) fn pop_console_key_event(
        &mut self,
        line: SourceLocation,
    ) -> Result<ConsoleKeyEvent, VmError> {
        const KEY: &str = "Std.Console.KeyEvent";
        match self.pop(line)? {
            Value::Record(record) if record.type_name == KEY || record.type_name == "<record>" => {
                Self::console_key_event_from_fields(&record.fields, line)
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
                .find(|(key, _)| key == name)
                .map(|(_, value)| value)
                .ok_or_else(|| {
                    internal_error(
                        format!("Std.Console.KeyEvent missing field `{name}`"),
                        "This indicates a compiler/runtime mismatch.",
                        line,
                    )
                })
        };

        let kind = match field("kind")? {
            Value::Integer(value) if *value >= 0 => *value as usize,
            _ => {
                return Err(internal_error(
                    "Std.Console.KeyEvent.kind must be a non-negative integer",
                    "This indicates a compiler/runtime mismatch.",
                    line,
                ));
            }
        };
        let ch = match field("ch")? {
            Value::Str(value) => key_event_char_from_string(value),
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
                Value::Boolean(value) => Ok(*value),
                _ => Err(internal_error(
                    format!("Std.Console.KeyEvent.{name} must be a boolean"),
                    "This indicates a compiler/runtime mismatch.",
                    line,
                )),
            }
        };

        Ok(ConsoleKeyEvent::new(
            kind,
            ch,
            read_bool("shift")?,
            read_bool("ctrl")?,
            read_bool("alt")?,
            read_bool("meta")?,
        ))
    }
}

fn key_event_char_value(ch: char) -> Value {
    if ch == '\0' {
        Value::Str(String::new().into())
    } else {
        Value::Str(ch.to_string().into())
    }
}

fn key_event_char_from_string(value: &str) -> char {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (None, _) => '\0',
        (Some(ch), None) => ch,
        (Some(ch), _) => ch,
    }
}
