//! Turbo Vision upstream `message_box` bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/message-box.md`

use super::try2::try2_message_box;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;

impl Worker {
    /// Show an upstream Turbo Vision message box and push the closing command id.
    pub(super) fn turbo_vision_message_box(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let options = u16::try_from(self.pop_int(line)?).map_err(|_| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.MessageBox options must fit in 16 bits",
                "Pass a non-negative options value such as `MessageBoxOption.About + MessageBoxOption.OkButton`.",
                line,
            )
        })?;
        let message = self.pop_turbo_vision_string("MessageBox Message", line)?;
        self.pop_tui_application(line)?;

        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.MessageBox(App, ...) must run on the main task",
                "Call `Application.MessageBox` from the main program, not from a `go` task.",
                line,
            ));
        }

        if self.with_tui(|tui| tui.session.is_headless()) {
            if let Some(command) = self.with_tui(|tui| tui.turbo_vision.test_dialog_result.take()) {
                return self.push(Value::Integer(command));
            }
        }

        if !self.try2.is_open() {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.MessageBox requires an open try-2 session",
                "Call `Application.Open` or `Application.OpenForTest` before `Application.MessageBox`.",
                line,
            ));
        }

        let command = try2_message_box(self, message, options, line)?;
        self.push(Value::Integer(command))
    }
}
