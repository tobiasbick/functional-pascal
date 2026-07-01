//! Headless native TUI testing intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

const PER_EVENT_SPINS: usize = 64;
const PUMP_UNTIL_IDLE_MAX: usize = 4096;

impl Worker {
    /// Executes headless native TUI testing intrinsics.
    pub(super) fn try_exec_tui_test_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let Intrinsic::Tui(code) = intrinsic else {
            return Ok(false);
        };

        match code {
            TuiIntrinsic::OpenForTest => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                let width = Self::test_dimension_to_u16(width, "Width", line)?;
                let height = Self::test_dimension_to_u16(height, "Height", line)?;

                self.with_console(|console| console.resize(width, height));
                self.reset_tui_host_state();
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.open_for_test(console, line))?;
                }
                self.push(Self::tui_application_record())?;
            }
            TuiIntrinsic::TestPump => {
                self.pop_tui_application(line)?;
                self.tui_test_pump_once(line)?;
            }
            TuiIntrinsic::TestPumpUntilIdle => {
                self.pop_tui_application(line)?;
                self.tui_test_pump_until_idle(line)?;
            }
            TuiIntrinsic::CloseForTest => {
                self.pop_tui_application(line)?;
                self.close_tui_application_state(line)?;
            }
            TuiIntrinsic::TestSetFileDialogResult => {
                self.turbo_vision_test_set_file_dialog_result(line)?;
            }
            TuiIntrinsic::TestSetDialogResult => {
                self.turbo_vision_test_set_dialog_result(line)?;
            }
            other if self.exec_tui_test_event_intrinsic(other, line)? => {}
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn test_dimension_to_u16(value: i64, name: &str, line: SourceLocation) -> Result<u16, VmError> {
        if value <= 0 || value > i64::from(u16::MAX) {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Application.OpenForTest({name}, …) requires {name} in 1..={}.",
                    u16::MAX
                ),
                "Pass positive screen dimensions, e.g. `Application.OpenForTest(80, 25)`.",
                line,
            ));
        }
        Ok(value as u16)
    }

    fn tui_test_pump_once(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.tui_host_ingest_console_events(PER_EVENT_SPINS, line)?;
        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let _ = tui.host.flush_pending_resize();
        }
        let _process_outcome = self.tui_host_process_next_inner(1, line)?;
        let _redraw_tag = self.tui_host_dispatch_redraw_inner(line)?;
        Ok(())
    }

    fn tui_test_pump_until_idle(&mut self, line: SourceLocation) -> Result<(), VmError> {
        for _ in 0..PUMP_UNTIL_IDLE_MAX {
            self.tui_host_ingest_console_events(PER_EVENT_SPINS, line)?;
            let flushed = {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.host.flush_pending_resize()
            };
            let process_outcome = self.tui_host_process_next_inner(1, line)?;
            let redraw_tag = self.tui_host_dispatch_redraw_inner(line)?;
            let input_pending = self
                .with_console_and_key_input(|_console, key_input| key_input.event_pending(line))?;
            if !process_outcome.did_work() && redraw_tag == 0 && !flushed && !input_pending {
                break;
            }
        }
        Ok(())
    }

    fn tui_host_ingest_console_events(
        &mut self,
        max_spins: usize,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        for _ in 0..max_spins.max(1) {
            let polled = {
                let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                self.with_console_and_key_input(|console, key_input| {
                    tui.session.poll_ui_event_all(console, key_input, line)
                })?
            };
            let Some(event) = polled else {
                break;
            };
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.host.ingest_ui_event(event);
        }
        Ok(())
    }
}
