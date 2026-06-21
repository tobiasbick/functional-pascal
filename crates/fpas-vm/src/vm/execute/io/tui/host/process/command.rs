//! Resolve and dispatch application commands.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{CommandEvent, ProcessOutcome};

impl Worker {
    /// Resolves a key against `HostBindCommandToActiveModal` for the top modal frame.
    pub(super) fn resolve_tui_modal_command(
        &self,
        key: &fpas_std::ConsoleKeyEvent,
    ) -> Option<CommandEvent> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.modals
            .resolve_active_command(key)
            .map(|command_id| CommandEvent::application(command_id, tui.modals.active_root_view()))
    }

    /// Resolves view-local and global command bindings (not modal-local).
    pub(super) fn resolve_tui_scoped_command(
        &self,
        key: &fpas_std::ConsoleKeyEvent,
    ) -> Option<CommandEvent> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(focused) = tui.views.focused_id() {
            for view_id in tui.views.ancestors_inclusive(focused) {
                if let Some(commands) = tui.view_commands.get(&view_id)
                    && let Some(command_id) = commands.resolve(key)
                {
                    return Some(CommandEvent::application(command_id, Some(view_id)));
                }
            }
        }

        tui.commands
            .resolve(key)
            .map(|id| CommandEvent::application(id, None))
    }

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
