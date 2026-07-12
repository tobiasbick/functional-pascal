//! Turbo Vision command queue and pump bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_on_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.register_tui_handler(
            2,
            "OnCommand",
            "Pass a `procedure (Application, integer)` command handler.",
            |tui, function| tui.on_command = Some(function),
            line,
        )?;
        Ok(())
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_on_key(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.register_tui_handler(
            2,
            "OnKey",
            "Pass a `function (Application, Std.Console.KeyEvent): boolean` handler.",
            |tui, function| tui.turbo_vision_on_key = Some(function),
            line,
        )?;
        Ok(())
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_on_mouse(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.register_tui_handler(
            2,
            "OnMouse",
            "Pass a `procedure (Application, Std.Console.Event)` mouse handler.",
            |tui, function| tui.turbo_vision_on_mouse = Some(function),
            line,
        )?;
        Ok(())
    }

    /// Queue a menu item command for headless tests.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_test_dispatch_menu_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let item_index = usize::try_from(self.pop_int(line)?).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Menu item index must be non-negative",
                "Pass a zero-based item index into the selected menu.",
                line,
            )
        })?;
        let menu_index = usize::try_from(self.pop_int(line)?).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Menu index must be non-negative",
                "Pass a zero-based top-level menu index.",
                line,
            )
        })?;
        let menu_bar_handle = self.pop_turbo_vision_menu_bar_handle(line)?;
        self.pop_tui_application(line)?;

        super::testing::bridge_test_dispatch_menu_command(
            self,
            menu_bar_handle,
            menu_index,
            item_index,
            line,
        )
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_quit(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.pop_tui_application(line)?;
        self.with_tui(|tui| tui.quit_requested = true);
        Ok(())
    }
}
