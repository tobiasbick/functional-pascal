//! `Std.Tui` VM execution helpers.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use super::super::super::Worker;
use super::super::super::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use super::super::super::runtime_error;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent};
use std::time::Instant;

const TUI_APPLICATION_TYPE: &str = "Std.Tui.Application";
const TUI_SIZE_TYPE: &str = "Std.Tui.Size";
const TUI_EVENT_TYPE: &str = "Std.Tui.Event";
const TUI_KEY_EVENT_TYPE: &str = "Std.Tui.KeyEvent";

impl Worker {
    pub(super) fn try_exec_tui_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::TuiApplicationOpen => {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.is_open = true;
                tui.redraw_pending = false;
                drop(tui);
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::TuiApplicationClose => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.is_open = false;
                tui.redraw_pending = false;
            }
            Intrinsic::TuiApplicationSize => {
                self.pop_tui_application(line)?;
                let (width, height) = self.with_console(|c| (c.screen_width(), c.screen_height()));
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::TuiApplicationReadEvent => {
                self.pop_tui_application(line)?;
                let event = self.read_tui_event(line)?;
                self.push(event)?;
            }
            Intrinsic::TuiApplicationReadEventTimeout => {
                let timeout_ms = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let event = self.read_tui_event_timeout(timeout_ms, line)?;
                match event {
                    Some(event) => self.push(Value::OptionSome(Box::new(event)))?,
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationPollEvent => {
                self.pop_tui_application(line)?;
                let event = self.poll_tui_event(line)?;
                match event {
                    Some(event) => self.push(Value::OptionSome(Box::new(event)))?,
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationRequestRedraw => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.redraw_pending = true;
            }
            Intrinsic::TuiApplicationRedrawPending => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                let pending = tui.redraw_pending;
                tui.redraw_pending = false;
                drop(tui);
                self.push(Value::Boolean(pending))?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn pop_tui_application(&mut self, line: SourceLocation) -> Result<(), VmError> {
        match self.pop(line)? {
            Value::Record { type_name, .. } if type_name == TUI_APPLICATION_TYPE => Ok(()),
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!("Expected {TUI_APPLICATION_TYPE}, got {}", other.type_name()),
                "Pass the value returned by Std.Tui.Application.Open().",
                line,
            )),
        }
    }

    fn read_tui_event(&mut self, line: SourceLocation) -> Result<Value, VmError> {
        loop {
            let event = self.with_key_input(|k| k.read_event(line))?;
            self.maybe_resize_on_console_event(&event);
            if let Some(value) = Self::map_console_event_to_tui(event) {
                return Ok(value);
            }
        }
    }

    fn read_tui_event_timeout(
        &mut self,
        timeout_ms: i64,
        line: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        let deadline = Instant::now() + std::time::Duration::from_millis(timeout_ms.max(0) as u64);

        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }

            let remaining = deadline
                .duration_since(now)
                .as_millis()
                .min(i64::MAX as u128) as i64;
            let event = self.with_key_input(|k| k.read_event_timeout(remaining, line))?;
            match event {
                Some(event) => {
                    self.maybe_resize_on_console_event(&event);
                    if let Some(value) = Self::map_console_event_to_tui(event) {
                        return Ok(Some(value));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn poll_tui_event(&mut self, line: SourceLocation) -> Result<Option<Value>, VmError> {
        loop {
            let event = self.with_key_input(|k| k.poll_event(line))?;
            match event {
                Some(event) => {
                    self.maybe_resize_on_console_event(&event);
                    if let Some(value) = Self::map_console_event_to_tui(event) {
                        return Ok(Some(value));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn maybe_resize_on_console_event(&self, event: &ConsoleEvent) {
        if event.kind == fpas_std::event_kind_index("Resize") {
            self.with_console(|c| c.resize(event.width as u16, event.height as u16));
        }
    }

    fn map_console_event_to_tui(event: ConsoleEvent) -> Option<Value> {
        if event.kind == fpas_std::event_kind_index("Key") {
            return Some(Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(0)),
                    ("key".into(), Self::tui_key_event_record(event.key)),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            });
        }

        if event.kind == fpas_std::event_kind_index("Resize") {
            return Some(Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(1)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    (
                        "size".into(),
                        Self::tui_size_record(event.width, event.height),
                    ),
                ],
            });
        }

        None
    }

    fn tui_application_record() -> Value {
        Value::Record {
            type_name: TUI_APPLICATION_TYPE.into(),
            fields: vec![],
        }
    }

    fn tui_size_record(width: i64, height: i64) -> Value {
        Value::Record {
            type_name: TUI_SIZE_TYPE.into(),
            fields: vec![
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
            ],
        }
    }

    fn tui_unknown_key_event() -> Value {
        Self::tui_key_event_record(ConsoleKeyEvent::new(
            fpas_std::key_event::key_kind_index("Unknown"),
            '\0',
            false,
            false,
            false,
            false,
        ))
    }

    fn tui_key_event_record(event: ConsoleKeyEvent) -> Value {
        Value::Record {
            type_name: TUI_KEY_EVENT_TYPE.into(),
            fields: vec![
                ("kind".into(), Value::Integer(event.kind as i64)),
                ("ch".into(), Value::Char(event.ch)),
                ("shift".into(), Value::Boolean(event.shift)),
                ("ctrl".into(), Value::Boolean(event.ctrl)),
                ("alt".into(), Value::Boolean(event.alt)),
                ("meta".into(), Value::Boolean(event.meta)),
            ],
        }
    }
}
