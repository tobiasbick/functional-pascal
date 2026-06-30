//! Resolve and dispatch application commands.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{CommandEvent, ProcessOutcome};

impl Worker {
    /// Invokes the registered application command handler for a resolved command.
    pub(in crate::vm::execute::io) fn dispatch_tui_command(
        &mut self,
        command: CommandEvent,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_command.clone()
        };
        let Some(handler) = handler else {
            return Ok(ProcessOutcome::Command { handled: false });
        };

        let app_rec = Self::tui_application_record();
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Value::Integer(command.id.0)],
            line,
        )?;
        Ok(ProcessOutcome::Command { handled: true })
    }
}
