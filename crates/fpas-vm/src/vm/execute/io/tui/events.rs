//! `Std.Tui` event-reading intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};
use fpas_std::{TuiEvent, tui_event_from_ui_event};

impl Worker {
    /// Executes event-reading `Std.Tui` intrinsics.
    pub(super) fn try_exec_tui_event_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::ApplicationReadEvent) => {
                self.pop_tui_application(line)?;
                let event = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console_and_key_input(|console, key_input| {
                        tui.session.read_event(console, key_input, line)
                    })?
                };
                self.push(Self::tui_event_record(event))?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationReadEventTimeout) => {
                let timeout_ms = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                loop {
                    let event = {
                        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        self.with_console_and_key_input(|console, key_input| {
                            tui.session
                                .read_event_timeout(console, key_input, timeout_ms, line)
                        })?
                    };
                    match event {
                        None => {
                            self.push(Value::OptionNone)?;
                            break;
                        }
                        Some(TuiEvent::Paste(_) | TuiEvent::FocusGained | TuiEvent::FocusLost) => {
                            continue;
                        }
                        Some(event) => {
                            self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?;
                            break;
                        }
                    }
                }
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationPollEvent) => {
                self.pop_tui_application(line)?;
                loop {
                    let event = {
                        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        self.with_console_and_key_input(|console, key_input| {
                            tui.session.poll_event(console, key_input, line)
                        })?
                    };
                    match event {
                        None => {
                            self.push(Value::OptionNone)?;
                            break;
                        }
                        Some(TuiEvent::Paste(_) | TuiEvent::FocusGained | TuiEvent::FocusLost) => {
                            continue;
                        }
                        Some(event) => {
                            self.push(Value::OptionSome(Box::new(Self::tui_event_record(event))))?;
                            break;
                        }
                    }
                }
            }
            Intrinsic::Tui(TuiIntrinsic::HostPollNext) => {
                self.pop_tui_application(line)?;
                let mapped = {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(ev) = tui.host.pop_ready_event() {
                        Some(ev)
                    } else {
                        let polled = self.with_console_and_key_input(|console, key_input| {
                            tui.session.poll_event_all(console, key_input, line)
                        })?;
                        if let Some(tui_event) = polled {
                            tui.host.ingest_tui_event(tui_event);
                        }
                        tui.host.pop_ready_event()
                    }
                };
                match mapped {
                    None => self.push(Value::OptionNone)?,
                    Some(ev) => {
                        let Some(tui_event) = tui_event_from_ui_event(ev) else {
                            self.push(Value::OptionNone)?;
                            return Ok(true);
                        };
                        self.push(Value::OptionSome(Box::new(Self::tui_event_record(
                            tui_event,
                        ))))?;
                    }
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
