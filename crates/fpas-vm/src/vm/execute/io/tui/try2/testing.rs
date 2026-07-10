//! Try-2 headless test helpers (`Application.TestClickButton`, `TestDispatchMenuCommand`).
//!
//! **Documentation:** `docs/refactor-tui-try-2/testing-strategy.md`

use super::headless::try2_ensure_headless_app;
use super::registry::ViewKind;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Queues a menu item command from try-2 menu bar state for headless tests.
pub(in crate::vm::execute::io::tui) fn try2_test_dispatch_menu_command(
    worker: &mut Worker,
    menu_bar_handle: u32,
    menu_index: usize,
    item_index: usize,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(menu_bar_handle, ViewKind::MenuBar)
        .map_err(|error| menu_bar_error(error, menu_bar_handle, line))?;

    let Some(command) = worker
        .try2
        .menu_item_command_id(menu_bar_handle, menu_index, item_index)
    else {
        return Err(menu_item_error(menu_index, item_index, line));
    };

    if !worker.with_tui(|tui| tui.session.is_headless()) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Application.TestDispatchMenuCommand is only supported in headless `Application.OpenForTest` runs",
            "Call `Application.OpenForTest` before `Application.TestDispatchMenuCommand`.",
            line,
        ));
    }

    try2_ensure_headless_app(worker, line)?;
    let Some(app) = worker.headless_tv_app.as_ref() else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Headless Turbo Vision session is not initialized",
            "Call `Application.OpenForTest` before `Application.TestDispatchMenuCommand`.",
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
            format!("MenuBar handle {handle} is not registered in the try-2 session"),
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
            format!("Button handle {handle} is not registered in the try-2 session"),
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
    /// Handles `Application.TestClickButton` on the try-2 headless path.
    pub(in crate::vm::execute::io::tui) fn exec_test_click_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        self.pop_tui_application(line)?;
        self.try2_queue_button_click(button_handle, line)
    }

    fn try2_queue_button_click(
        &mut self,
        button_handle: u32,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.try2
            .registry
            .require(button_handle, ViewKind::Button)
            .map_err(|error| button_handle_error(error, button_handle, line))?;

        if !self.with_tui(|tui| tui.session.is_headless()) {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.TestClickButton is only supported in headless `Application.OpenForTest` runs",
                "Call `Application.OpenForTest` before `Application.TestClickButton`.",
                line,
            ));
        }

        let Some(point) = self.try2.button_click_point(button_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Button handle {button_handle} has no try-2 click target"),
                "Use a button attached via `Dialog.Add` or `Window.Add` in the active try-2 session.",
                line,
            ));
        };

        try2_ensure_headless_app(self, line)?;
        let Some(app) = self.headless_tv_app.as_ref() else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before `Application.TestClickButton`.",
                line,
            ));
        };

        app.push_mouse_down(point.x, point.y);
        app.push_mouse_up(point.x, point.y);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::execute::io::tui::try2::views::{try2_dialog_add_button, try2_dialog_new_modal};
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_OK;
    use turbo_vision::core::geometry::Rect;

    #[test]
    fn button_click_point_is_registered_on_add() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog = try2_dialog_new_modal(
            &mut worker,
            Rect::from_coords(5, 3, 30, 8),
            "Test".into(),
            loc(),
        )
        .expect("dialog");
        let button = try2_dialog_add_button(
            &mut worker,
            dialog,
            Rect::from_coords(10, 4, 20, 6),
            "OK".into(),
            CM_OK,
            true,
            loc(),
        )
        .expect("button");
        let point = worker.try2.button_click_point(button).expect("click point");
        assert_eq!(point.x, 25);
        assert_eq!(point.y, 10);
    }
}
