//! Turbo Vision application-level callback and test-pump operations.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (Turbo Vision spike API).

use super::widgets::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::ProcessOutcome;
use turbo_vision::core::event::Event;

impl Worker {
    pub(super) fn turbo_vision_register_on_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.register_tui_handler(
            2,
            "OnCommand",
            "Pass a `procedure (Application, integer)` command handler.",
            |tui, function| tui.on_command = Some(function),
            line,
        )?;
        Ok(())
    }

    pub(super) fn turbo_vision_test_click_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            let Some(TurboVisionObject::Button(button)) =
                tui.turbo_vision.objects.get(&button_handle)
            else {
                return Err(unknown_handle_error("Button", button_handle, line));
            };
            tui.turbo_vision
                .pending_commands
                .push_back(button.command_id);
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn turbo_vision_pump(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.pop_tui_application(line)?;
        let command = self.with_tui(|tui| {
            if tui.turbo_vision.quit_requested {
                None
            } else {
                tui.turbo_vision.pending_commands.pop_front()
            }
        });

        let Some(command) = command else {
            self.push(Value::Integer(ProcessOutcome::Idle.bridge_tag()))?;
            return Ok(());
        };

        let outcome = self
            .dispatch_turbo_vision_command_event(&Event::command(command), line)?
            .unwrap_or(ProcessOutcome::Idle);
        self.push(Value::Integer(outcome.bridge_tag()))
    }

    pub(super) fn turbo_vision_quit(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            tui.quit_requested = true;
            tui.turbo_vision.quit_requested = true;
        });
        Ok(())
    }
}
