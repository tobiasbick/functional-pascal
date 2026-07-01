//! FPAS callback invocation from Turbo Vision command events.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{CommandEvent, CommandId, ProcessOutcome};
use turbo_vision::core::event::{Event, EventType};

use super::command_map::turbo_vision_command_to_fpas;

impl Worker {
    /// Dispatch a Turbo Vision command event through the registered FPAS `OnCommand` callback.
    ///
    /// Non-command events return `Ok(None)` so callers can keep routing keyboard and mouse events
    /// through the future Turbo Vision event pump.
    pub(in crate::vm::execute::io) fn dispatch_turbo_vision_command_event(
        &mut self,
        event: &Event,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        if event.what != EventType::Command {
            return Ok(None);
        }

        let fpas_command = turbo_vision_command_to_fpas(event.command);
        let command = CommandEvent::application(CommandId(i64::from(fpas_command)), None);
        self.dispatch_tui_command(command, line).map(Some)
    }

    /// Test hook for the internal Turbo Vision command bridge.
    #[cfg(test)]
    pub(crate) fn dispatch_turbo_vision_command_event_for_tests(
        &mut self,
        event: &Event,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        self.dispatch_turbo_vision_command_event(event, line)
    }
}
