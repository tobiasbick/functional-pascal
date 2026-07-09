//! Turbo Vision modal file dialog bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::try2::try2_run_file_dialog;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::path::PathBuf;
use turbo_vision::views::file_dialog::FileDialog;

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

        if self.try2.is_open() {
            let selected = try2_run_file_dialog(self, bounds, title, wildcard, start_path, line)?;
            return self.push_optional_string(selected);
        }

        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.RunFileDialog(App, ...) must run on the main task",
                "Call `Application.RunFileDialog` from the main program, not from a `go` task.",
                line,
            ));
        }

        if self.with_tui(|tui| tui.session.is_headless()) {
            let result = self.with_tui(|tui| tui.turbo_vision.test_file_dialog_result.take());
            return self.push_optional_string(result.unwrap_or(None));
        }

        let initial_dir = start_path
            .filter(|path| !path.is_empty())
            .map(PathBuf::from);
        let mut file_dialog = FileDialog::new(bounds, &title, &wildcard, initial_dir).build();
        let selected = self.turbo_vision_with_live_app(line, |app| Ok(file_dialog.execute(app)))?;
        self.push_optional_string(selected.map(|path| path.to_string_lossy().into_owned()))
    }

    /// Queue the result consumed by the next headless `Application.RunFileDialog` call.
    pub(super) fn turbo_vision_test_set_file_dialog_result(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let result = self.pop_optional_string("FileDialog test result", line)?;
        self.pop_tui_application(line)?;
        if self.try2.is_open() {
            self.try2.set_file_dialog_result(result);
            return Ok(());
        }
        self.with_tui(|tui| {
            tui.turbo_vision.test_file_dialog_result = Some(result);
        });
        Ok(())
    }
}
