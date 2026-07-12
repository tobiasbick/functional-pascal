//! Turbo Vision bridge headless test helpers (`Test.Click`, `Test.DispatchMenu`, `Application.TestClickMouse`).
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use super::headless::bridge_ensure_headless_app;
use super::registry::ViewKind;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Queues a menu item command from Turbo Vision menu bar state for headless tests.
pub(in crate::vm::execute::io::tui) fn bridge_test_dispatch_menu_command(
    worker: &mut Worker,
    menu_bar_handle: u32,
    menu_index: usize,
    item_index: usize,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(menu_bar_handle, ViewKind::MenuBar)
        .map_err(|error| menu_bar_error(error, menu_bar_handle, line))?;

    let Some(command) = worker
        .bridge
        .menu_item_command_id(menu_bar_handle, menu_index, item_index)
    else {
        return Err(menu_item_error(menu_index, item_index, line));
    };

    if !worker.with_tui(|tui| tui.session.is_headless()) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Test.DispatchMenu is only supported in headless `Application.OpenForTest` runs",
            "Call `Application.OpenForTest` before `Test.DispatchMenu`.",
            line,
        ));
    }

    bridge_ensure_headless_app(worker, line)?;
    let Some(app) = worker.headless_tv_app.as_ref() else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Headless Turbo Vision session is not initialized",
            "Call `Application.OpenForTest` before `Test.DispatchMenu`.",
            line,
        ));
    };

    app.push_command(command);
    Ok(())
}

fn menu_bar_error(
    error: super::registry::RegistryError,
    _handle: u32,
    line: SourceLocation,
) -> VmError {
    let (message, help) = match error {
        super::registry::RegistryError::UnknownHandle(handle) => (
            format!("MenuBar handle {handle} is not registered in the Turbo Vision session"),
            "Use a handle returned by `MenuBar.New`.",
        ),
        super::registry::RegistryError::WrongKind { handle, .. } => (
            format!("Handle {handle} is not a MenuBar"),
            "Pass a handle from `MenuBar.New`.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}

fn menu_item_error(menu_index: usize, item_index: usize, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("Menu item ({menu_index}, {item_index}) is out of range or a separator"),
        "Use a menu and item index from the `MenuBar.New` data.",
        line,
    )
}

fn button_handle_error(
    error: super::registry::RegistryError,
    _handle: u32,
    line: SourceLocation,
) -> VmError {
    let (message, help) = match error {
        super::registry::RegistryError::UnknownHandle(handle) => (
            format!("Button handle {handle} is not registered in the Turbo Vision session"),
            "Use a handle returned by `Button.New`.",
        ),
        super::registry::RegistryError::WrongKind { handle, .. } => (
            format!("Handle {handle} is not a Button"),
            "Pass a handle from `Button.New`.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}

impl Worker {
    /// Handles `Test.Click` on the Turbo Vision headless path.
    pub(in crate::vm::execute::io::tui) fn exec_test_click_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        self.pop_tui_application(line)?;
        self.bridge_queue_button_click(button_handle, line)
    }

    fn bridge_queue_button_click(
        &mut self,
        button_handle: u32,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.bridge
            .registry
            .require(button_handle, ViewKind::Button)
            .map_err(|error| button_handle_error(error, button_handle, line))?;

        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Test.Click is only supported in headless `Application.OpenForTest` runs",
                "Call `Application.OpenForTest` before `Test.Click`.",
                line,
            ));
        }

        let Some(point) = self.bridge.button_click_point(button_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Button handle {button_handle} has no Turbo Vision click target"),
                "Use a button attached via `Dialog.Add` or `Window.Add` in the active Turbo Vision session.",
                line,
            ));
        };

        bridge_ensure_headless_app(self, line)?;
        let Some(app) = self.headless_tv_app.as_ref() else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before `Test.Click`.",
                line,
            ));
        };

        app.push_mouse_down(point.x, point.y);
        app.push_mouse_up(point.x, point.y);
        Ok(())
    }

    /// Queue a left mouse down at screen coordinates for headless `OpenForTest` runs.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_test_click_mouse(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let y = self.pop_int(line)?;
        let x = self.pop_int(line)?;
        self.pop_tui_application(line)?;

        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.TestClickMouse is only supported in headless `Application.OpenForTest` runs",
                "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                line,
            ));
        }

        let x = i16::try_from(x).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse X coordinate {x} is out of range for Turbo Vision"),
                "Use screen coordinates within the headless terminal size.",
                line,
            )
        })?;
        let y = i16::try_from(y).map_err(|_| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse Y coordinate {y} is out of range for Turbo Vision"),
                "Use screen coordinates within the headless terminal size.",
                line,
            )
        })?;

        self.bridge_test_click_mouse(x, y, line)
    }

    fn bridge_test_click_mouse(
        &mut self,
        x: i16,
        y: i16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let width = self.with_console(|console| console.screen_width() as u16);
        let height = self.with_console(|console| console.screen_height() as u16);
        if x < 0 || y < 0 || x >= width as i16 || y >= height as i16 {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Mouse coordinate ({x}, {y}) is outside the headless terminal"),
                "Use screen coordinates within the headless terminal size.",
                line,
            ));
        }

        self.turbo_vision_ensure_headless_app(width, height)
            .map_err(|error| {
                runtime_error(
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    format!("Headless Turbo Vision initialization failed: {error}"),
                    "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                    line,
                )
            })?;
        let target = self.bridge.mouse_hit_target_for_screen(x, y);
        let point = target
            .map(|target| target.click)
            .unwrap_or_else(|| turbo_vision::core::geometry::Point::new(x, y));
        if let Some(target) = target {
            self.bridge.queue_mouse_state_toggle(target.handle);
        }

        let mut app = self.headless_tv_app.take().ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before `Application.TestClickMouse`.",
                line,
            )
        })?;
        app.push_mouse_down(point.x, point.y);
        if app.desktop_mut().child_count() > 0 {
            let _ = app.dispatch_next_input_event();
        }
        app.push_mouse_up(point.x, point.y);
        if app.desktop_mut().child_count() > 0 {
            let _ = app.dispatch_next_input_event();
        }
        self.headless_tv_app = Some(app);

        Ok(())
    }

    /// Applies stateful headless mouse clicks after the test run loop has consumed input.
    pub(in crate::vm::execute::io::tui) fn apply_pending_mouse_state_toggles(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        for handle in self.bridge.take_pending_mouse_state_toggles() {
            if self
                .bridge
                .registry
                .require(handle, ViewKind::CheckBox)
                .is_ok()
            {
                let cell = self.bridge.check_box_cell(handle).ok_or_else(|| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("CheckBox handle {handle} has no host state"),
                        "Use a CheckBox attached to the active Turbo Vision session.",
                        line,
                    )
                })?;
                cell.set(!cell.read());
                continue;
            }
            let group_id = self
                .bridge
                .radio_button_state(handle)
                .ok_or_else(|| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("RadioButton handle {handle} has no host state"),
                        "Use a RadioButton attached to the active Turbo Vision session.",
                        line,
                    )
                })?
                .group_id;
            self.bridge
                .deselect_radio_group_except(group_id, Some(handle));
            let cell = self
                .bridge
                .radio_button_selected_cell(handle)
                .ok_or_else(|| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        format!("RadioButton handle {handle} has no host state"),
                        "Use a RadioButton attached to the active Turbo Vision session.",
                        line,
                    )
                })?;
            cell.set(true);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::execute::io::tui::bridge::views::{
        bridge_dialog_add_button, bridge_dialog_new_modal,
    };
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_OK;
    use turbo_vision::core::geometry::Rect;

    #[test]
    fn button_click_point_is_registered_on_add() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let dialog = bridge_dialog_new_modal(
            &mut worker,
            Rect::from_coords(5, 3, 30, 8),
            "Test".into(),
            loc(),
        )
        .expect("dialog");
        let button = bridge_dialog_add_button(
            &mut worker,
            dialog,
            Rect::from_coords(10, 4, 20, 6),
            "OK".into(),
            CM_OK,
            true,
            loc(),
        )
        .expect("button");
        let point = worker
            .bridge
            .button_click_point(button)
            .expect("click point");
        assert_eq!(point.x, 25);
        assert_eq!(point.y, 10);
    }
}
