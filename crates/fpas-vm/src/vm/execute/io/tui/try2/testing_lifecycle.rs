//! Try-2 headless Turbo Vision testing intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Worker {
    /// Executes headless Turbo Vision testing intrinsics.
    pub(in crate::vm::execute::io::tui) fn try_exec_tui_test_host_intrinsic(
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
                self.reset_tui_session_state();
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.open_for_test(console, line))?;
                }
                self.open_try2_session();
                self.push(Self::tui_application_record())?;
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
            _ => return Ok(false),
        }

        Ok(true)
    }

    /// Queue the closing command consumed by the next headless modal call.
    pub(super) fn turbo_vision_test_set_dialog_result(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let command = self.pop_int(line)?;
        self.pop_tui_application(line)?;
        self.try2.set_dialog_result(command);
        Ok(())
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
}
