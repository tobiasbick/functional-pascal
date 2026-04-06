//! `Std.Tui` VM execution helpers.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use super::super::super::Worker;
use super::super::super::diagnostics::{TYPE_MISMATCH_CODE, VmError};
use super::super::super::runtime_error;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_std::{ConsoleKeyEvent, TuiEvent};

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
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.open(console, key_input, line)
                    })?;
                }
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::TuiApplicationClose => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                self.with_console_and_key_input(|console, key_input| {
                    tui.session.close(console, key_input, line)
                })?;
            }
            Intrinsic::TuiApplicationSize => {
                self.pop_tui_application(line)?;
                let (width, height) = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.size(console, line))?
                };
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::TuiApplicationReadEvent => {
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.read_event(console, key_input, line)
                    })?
                };
                self.push(Self::tui_event_record(event))?;
            }
            Intrinsic::TuiApplicationReadEventTimeout => {
                let timeout_ms = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session
                            .read_event_timeout(console, key_input, timeout_ms, line)
                    })?
                };
                match event {
                    Some(event) => {
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?
                    }
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationPollEvent => {
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.poll_event(console, key_input, line)
                    })?
                };
                match event {
                    Some(event) => {
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?
                    }
                    None => self.push(Value::OptionNone)?,
                }
            }
            Intrinsic::TuiApplicationRequestRedraw => {
                self.pop_tui_application(line)?;
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.session.request_redraw(line)?;
            }
            Intrinsic::TuiApplicationRedrawPending => {
                self.pop_tui_application(line)?;
                let pending = {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    tui.session.take_redraw_pending(line)?
                };
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

    fn tui_event_record(event: TuiEvent) -> Value {
        match event {
            TuiEvent::Key(key) => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(0)),
                    ("key".into(), Self::tui_key_event_record(key)),
                    ("size".into(), Self::tui_size_record(0, 0)),
                ],
            },
            TuiEvent::Resize { width, height } => Value::Record {
                type_name: TUI_EVENT_TYPE.into(),
                fields: vec![
                    ("kind".into(), Value::Integer(1)),
                    ("key".into(), Self::tui_unknown_key_event()),
                    ("size".into(), Self::tui_size_record(width, height)),
                ],
            },
        }
    }
}
