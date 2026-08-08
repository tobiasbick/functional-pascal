//! Runtime conversion between console host events and FPAS records.

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use fpas_std::ConsoleKeyEvent;

impl Worker {
    pub(crate) fn key_event_record(
        &self,
        event: ConsoleKeyEvent,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        self.record_value(
            "Std.Console.KeyEvent",
            vec![
                Value::Integer(event.kind as i64),
                key_event_char_value(event.ch),
                Value::Boolean(event.shift),
                Value::Boolean(event.ctrl),
                Value::Boolean(event.alt),
                Value::Boolean(event.meta),
            ],
            location,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Console event records have a fixed field layout mirroring Std.Console.Event"
    )]
    fn console_event_record_fields(
        &self,
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
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        self.record_value(
            "Std.Console.Event",
            vec![
                Value::Integer(kind as i64),
                self.key_event_record(key, location)?,
                Value::Integer(mouse_action as i64),
                Value::Integer(mouse_button as i64),
                Value::Integer(mouse_x),
                Value::Integer(mouse_y),
                Value::Integer(width),
                Value::Integer(height),
                Value::Str(text.into()),
                Value::Boolean(shift),
                Value::Boolean(ctrl),
                Value::Boolean(alt),
                Value::Boolean(meta),
            ],
            location,
        )
    }

    /// Builds one `Std.Console.Event` record from the runtime console event model.
    pub(crate) fn console_event_record(
        &self,
        event: fpas_std::ConsoleEvent,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
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
        self.console_event_record_fields(
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
            location,
        )
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

pub(crate) fn console_key_event_from_value(
    value: &Value,
    line: SourceLocation,
) -> Result<ConsoleKeyEvent, VmError> {
    const KEY: &str = "Std.Console.KeyEvent";
    match value {
        Value::Record(record) if record.body().layout.type_name.eq_ignore_ascii_case(KEY) => {
            Worker::console_key_event_from_fields(&record_fields(record), line)
        }
        other => Err(runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected {KEY}, got {}", other.type_name()),
            "Pass a `Std.Console.KeyEvent` value.",
            line,
        )),
    }
}

fn record_fields(record: &fpas_bytecode::SharedRecord) -> Vec<(String, Value)> {
    record
        .body()
        .layout
        .fields
        .iter()
        .cloned()
        .zip(record.body().values.iter().cloned())
        .collect()
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

pub(crate) fn key_event_record(
    worker: &Worker,
    event: ConsoleKeyEvent,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.key_event_record(event, location)
}

pub(crate) fn console_event_record(
    worker: &Worker,
    event: fpas_std::ConsoleEvent,
    location: SourceLocation,
) -> Result<Value, VmError> {
    worker.console_event_record(event, location)
}
