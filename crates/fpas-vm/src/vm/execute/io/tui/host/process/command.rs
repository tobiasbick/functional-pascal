//! Resolve and dispatch application commands.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{CommandEvent, CommandKind, ProcessOutcome};

impl Worker {
    /// Resolves a key against `HostBindCommandToActiveModal` for the top modal frame.
    pub(super) fn resolve_tui_modal_command(
        &self,
        key: &fpas_std::ConsoleKeyEvent,
    ) -> Option<CommandEvent> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.modals
            .resolve_active_command(key)
            .map(|command_id| CommandEvent::resolve(command_id, tui.modals.active_root_view()))
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
                    return Some(CommandEvent::resolve(command_id, Some(view_id)));
                }
            }
        }

        tui.commands
            .resolve(key)
            .map(|id| CommandEvent::resolve(id, None))
    }

    /// Invokes the registered application command handler for a resolved command.
    pub(in crate::vm::execute::io) fn dispatch_tui_command(
        &mut self,
        command: CommandEvent,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        if let Some(outcome) = self.try_dispatch_builtin_command(command, line)? {
            return Ok(outcome);
        }
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

    fn try_dispatch_builtin_command(
        &mut self,
        command: CommandEvent,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        match command.kind {
            CommandKind::Application | CommandKind::Accept | CommandKind::Cancel => Ok(None),
            CommandKind::Close => {
                let handled = self.dispatch_close_command(command, line)?;
                Ok(Some(ProcessOutcome::Command { handled }))
            }
            CommandKind::NextWindow => {
                let (changed, previous, current) = self.with_tui(|tui| {
                    let exclude = tui
                        .modals
                        .active_root_view()
                        .into_iter()
                        .collect::<Vec<_>>();
                    let previous = tui.views.focused_id();
                    let activation = tui.views.activate_next_root_excluding(&exclude);
                    (activation.is_some(), previous, tui.views.focused_id())
                });
                if changed {
                    self.request_focus_transition_redraw(previous, current, line)?;
                    self.invoke_focus_transition(previous.is_some(), line)?;
                }
                Ok(Some(ProcessOutcome::Command { handled: changed }))
            }
            CommandKind::Zoom | CommandKind::ZoomBack => {
                Ok(Some(ProcessOutcome::Command { handled: false }))
            }
        }
    }

    fn dispatch_close_command(
        &mut self,
        command: CommandEvent,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let root = command
            .source
            .and_then(|id| self.with_tui(|tui| tui.views.root_of(id)));
        let Some(root) = root else {
            return Ok(false);
        };

        let is_active_modal = self.with_tui(|tui| tui.modals.active_root_view() == Some(root));
        if is_active_modal {
            self.with_tui(|tui| Self::close_active_modal(tui, line));
            return Ok(true);
        }

        Ok(false)
    }
}
