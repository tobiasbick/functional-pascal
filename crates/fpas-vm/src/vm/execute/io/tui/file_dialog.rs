//! Turbo Vision modal file dialog bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::try2::try2_run_file_dialog;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Worker {
    /// Run a modal Turbo Vision file dialog and push `Option of string`.
    pub(super) fn turbo_vision_run_file_dialog(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let start_path = self.pop_optional_string("FileDialog StartPath", line)?;
        let wildcard = self.pop_turbo_vision_string("FileDialog Wildcard", line)?;
        let title = self.pop_turbo_vision_string("FileDialog Title", line)?;
        let bounds = self.pop_turbo_vision_rect(line)?;
        self.pop_tui_application(line)?;

        if !self.try2.is_open() {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.RunFileDialog requires an open try-2 session",
                "Call `Application.Open` or `Application.OpenForTest` before `Application.RunFileDialog`.",
                line,
            ));
        }

        let selected = try2_run_file_dialog(self, bounds, title, wildcard, start_path, line)?;
        self.push_optional_string(selected)
    }

    /// Queue the result consumed by the next headless `Application.RunFileDialog` call.
    pub(super) fn turbo_vision_test_set_file_dialog_result(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let result = self.pop_optional_string("FileDialog test result", line)?;
        self.pop_tui_application(line)?;
        self.try2.set_file_dialog_result(result);
        Ok(())
    }
}
