//! Runtime conversion between console host events and FPAS records.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
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
}

fn key_event_char_value(ch: char) -> Value {
    if ch == '\0' {
        Value::Str(String::new().into())
    } else {
        Value::Str(ch.to_string().into())
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
