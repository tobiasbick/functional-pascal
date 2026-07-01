//! Turbo Vision command queue and pump bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::ProcessOutcome;
use turbo_vision::core::event::Event;

impl Worker {
    pub(super) fn turbo_vision_register_on_command(
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

    pub(super) fn turbo_vision_register_on_key(
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

    pub(super) fn turbo_vision_register_on_mouse(
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

    pub(super) fn turbo_vision_test_click_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
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

    /// Queue a menu item command for headless tests.
    pub(super) fn turbo_vision_test_dispatch_menu_command(
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

        self.with_tui(|tui| {
            let Some(TurboVisionObject::MenuBar(menu_bar)) =
                tui.turbo_vision.objects.get(&menu_bar_handle)
            else {
                return Err(unknown_handle_error("MenuBar", menu_bar_handle, line));
            };
            let Some(menu) = menu_bar.menus.get(menu_index) else {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Menu index {menu_index} is out of range"),
                    "Use a menu index returned by `Application.CreateMenuBar`.",
                    line,
                ));
            };
            let Some(item) = menu.items.get(item_index) else {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Menu item index {item_index} is out of range"),
                    "Use an item index inside the selected menu.",
                    line,
                ));
            };
            if item.command_id == 0 {
                return Err(runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Menu separators cannot be dispatched",
                    "Select a menu item with a non-zero `commandId`.",
                    line,
                ));
            }
            tui.turbo_vision.pending_commands.push_back(item.command_id);
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn turbo_vision_pump(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.pop_tui_application(line)?;
        let outcome = self.turbo_vision_pump_next_command(line)?;
        if self.with_tui(|tui| tui.session.is_headless()) {
            self.turbo_vision_reconcile_after_step(None, line)?;
        }
        self.push(Value::Integer(outcome.bridge_tag()))
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_pump_next_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        let command = self.with_tui(|tui| {
            if tui.turbo_vision.quit_requested {
                None
            } else {
                tui.turbo_vision.pending_commands.pop_front()
            }
        });

        let Some(command) = command else {
            return Ok(ProcessOutcome::Idle);
        };

        Ok(self
            .dispatch_turbo_vision_command_event(&Event::command(command), line)?
            .unwrap_or(ProcessOutcome::Idle))
    }

    pub(super) fn turbo_vision_quit(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            tui.quit_requested = true;
            tui.turbo_vision.quit_requested = true;
        });
        Ok(())
    }
}
