//! Resolve and dispatch application commands.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::CommandId;

impl Worker {
    /// Resolves a key through focused-view ancestors, the active modal, and global bindings.
    pub(super) fn resolve_tui_command(&self, key: &fpas_std::ConsoleKeyEvent) -> Option<CommandId> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(focused) = tui.views.focused_id() {
            for view_id in tui.views.ancestors_inclusive(focused) {
                if let Some(commands) = tui.view_commands.get(&view_id)
                    && let Some(command_id) = commands.resolve(key)
                {
                    return Some(command_id);
                }
            }
        }

        if let Some(command_id) = tui.modals.resolve_active_command(key) {
            return Some(command_id);
        }

        tui.commands.resolve(key)
    }

    /// Invokes the registered application command handler for a resolved command.
    pub(in crate::vm::execute::io) fn dispatch_tui_command(
        &mut self,
        command_id: CommandId,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_command.clone()
        };
        let Some(handler) = handler else {
            return Ok(17);
        };

        let app_rec = Self::tui_application_record();
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Value::Integer(command_id.0)],
            line,
        )?;
        Ok(16)
    }
}
