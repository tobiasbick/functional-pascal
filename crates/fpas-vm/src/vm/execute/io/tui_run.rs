//! Turbo Vision `Application.Run` execution.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    pub(super) fn tui_application_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.pop_tui_application(line)?;
        if !self.with_tui(|tui| !tui.turbo_vision.objects.is_empty()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) requires at least one Turbo Vision widget",
                "Create a dialog, window, or control with `Application.Create*` before calling `Application.Run(App)`. For simple terminal apps use `Std.Console` directly.",
                line,
            ));
        }

        let run_result = self.turbo_vision_application_run(line);
        let close_result = self.close_tui_application_state(line);

        match (run_result, close_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }
}
