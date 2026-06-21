//! `Std.Tui` view, modal, command-binding, and widget intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod commands;
mod handles;
mod modal;
mod tree;
mod widgets;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

impl Worker {
    /// Executes `Std.Tui` view, modal, command-binding, and widget intrinsics.
    pub(super) fn try_exec_tui_view_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let Intrinsic::Tui(intrinsic) = intrinsic else {
            return Ok(false);
        };

        if self.try_exec_tui_modal_intrinsic(intrinsic, line)?
            || self.try_exec_tui_control_intrinsic(intrinsic, line)?
            || self.try_exec_tui_command_binding_intrinsic(intrinsic, line)?
            || self.try_exec_tui_view_tree_intrinsic(intrinsic, line)?
            || self.try_exec_tui_view_widget_intrinsic(intrinsic, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }
}
