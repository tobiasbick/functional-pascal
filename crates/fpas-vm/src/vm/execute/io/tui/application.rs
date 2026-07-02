//! `Std.Tui` application lifecycle intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};

impl Worker {
    /// Executes application-level `Std.Tui` intrinsics.
    pub(super) fn try_exec_tui_application_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::ApplicationOpen) => {
                self.reset_tui_session_state();
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.open_deferred(console, line))?;
                }
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationClose) => {
                self.pop_tui_application(line)?;
                self.close_tui_application_state(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationRun) => {
                self.tui_application_run(line)?;
                self.push(Value::Unit)?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationSize) => {
                self.pop_tui_application(line)?;
                let (width, height) = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.size(console, line))?
                };
                self.push(Self::tui_size_record(width, height))?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
