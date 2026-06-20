//! View, modal, and application command bindings.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, TuiIntrinsic};

impl Worker {
    /// Executes application, view-local, and active-modal command-binding intrinsics.
    pub(super) fn try_exec_tui_command_binding_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::HostBindCommand => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    tui.commands.bind(key, fpas_std::CommandId(command_id));
                });
            }
            TuiIntrinsic::HostBindCommandToView => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                let view_id = self.pop_tui_view_id(line)?;
                self.pop_tui_application(line)?;
                self.require_registered_tui_view(view_id, line)?;
                self.with_tui(|tui| {
                    tui.view_commands
                        .entry(view_id)
                        .or_default()
                        .bind(key, fpas_std::CommandId(command_id));
                });
            }
            TuiIntrinsic::HostBindCommandToActiveModal => {
                let command_id = self.pop_int(line)?;
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| {
                    let _ = tui
                        .modals
                        .bind_command_to_active(key, fpas_std::CommandId(command_id));
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
