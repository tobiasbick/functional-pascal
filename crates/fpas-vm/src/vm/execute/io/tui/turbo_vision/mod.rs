//! Turbo Vision bridge internals for the planned `Std.Tui` rewrite.
//!
//! Planned API work is tracked in `docs/future/turbo-vision-4-rust/04-implementation-phases.md`.

mod application;
mod commands;
mod handles;
mod widgets;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};

impl Worker {
    /// Execute Turbo Vision backed `Std.Tui` spike intrinsics.
    pub(super) fn try_exec_turbo_vision_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::CreateDialog) => {
                self.turbo_vision_create_dialog(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateButton) => {
                self.turbo_vision_create_button(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::AddChild) => {
                self.turbo_vision_add_child(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand) => {
                self.turbo_vision_register_on_command(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::Pump) => {
                self.turbo_vision_pump(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::Quit) => {
                self.turbo_vision_quit(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::TestClickButton) => {
                self.turbo_vision_test_click_button(line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
