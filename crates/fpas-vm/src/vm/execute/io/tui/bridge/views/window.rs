//! Turbo Vision bridge `Window` construction and `Window.Add`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::button::button_click_point;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::bridge::session::TuiRoot;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::window::Window;

/// Creates a modeless window root in the Turbo Vision session (`Window.New`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_new(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(bridge_session_closed_error(line));
    }

    let window = Window::new(bounds, &title);
    Ok(worker
        .bridge
        .insert_root(TuiRoot::Window(Box::new(window)), ViewKind::Window))
}

/// Replaces a window title (`Window.SetTitle`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_set_title(
    worker: &mut Worker,
    handle: u32,
    title: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::Window)
        .map_err(|error| registry_error(error, line))?;

    let Some(TuiRoot::Window(window)) = worker.bridge.root_mut(handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {handle} is not an owned Turbo Vision root"),
            "Call `Window.SetTitle` before `Desktop.Add`.",
            line,
        ));
    };

    window.set_title(&title);
    Ok(())
}

/// Attaches a detached button to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_attach_button(
    worker: &mut Worker,
    window_handle: u32,
    button_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.bridge.is_on_desktop(window_handle) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {window_handle} is already on the desktop"),
            "Call `Window.Add` before `Desktop.Add`.",
            line,
        ));
    }

    worker
        .bridge
        .registry
        .require(window_handle, ViewKind::Window)
        .map_err(|error| registry_error(error, line))?;
    worker
        .bridge
        .registry
        .require(button_handle, ViewKind::Button)
        .map_err(|error| registry_error(error, line))?;

    let Some(detached) = worker.bridge.take_detached_button(button_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Button handle {button_handle} is not detached"),
            "Pass a handle from `Button.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let (view_id, click) = {
        let Some(TuiRoot::Window(window)) = worker.bridge.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        let window_bounds = window.bounds();
        let view_id = window.add(detached.button).as_u16();
        (
            view_id,
            button_click_point(window_bounds, detached.local_bounds),
        )
    };

    worker
        .bridge
        .registry
        .set_view_id(button_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(button_handle), line))?;
    worker.bridge.set_child_parent(button_handle, window_handle);
    worker.bridge.set_button_click_point(button_handle, click);
    Ok(())
}

fn bridge_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the Turbo Vision path.",
        line,
    )
}

fn registry_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("Handle {handle} is not live"),
            "Use a handle returned by `Window.New` in the active session.",
        ),
        RegistryError::WrongKind {
            handle,
            expected,
            actual,
        } => (
            format!("Handle {handle} expected {:?}, got {:?}", expected, actual),
            "Pass a Window handle as the parent.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use crate::vm::execute::io::tui::bridge::registry::ViewKind;
    use crate::vm::execute::io::tui::bridge::views::button::bridge_button_new;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_QUIT;

    #[test]
    fn new_registers_window_root() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let bounds = Rect::new(5, 3, 30, 10);
        let handle = bridge_window_new(&mut worker, bounds, "Test".into(), loc()).expect("window");
        assert_eq!(
            worker
                .bridge
                .registry
                .require(handle, ViewKind::Window)
                .unwrap()
                .kind,
            ViewKind::Window
        );
    }

    #[test]
    fn button_new_and_window_add_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let window = bridge_window_new(
            &mut worker,
            Rect::from_coords(5, 3, 30, 10),
            "Test".into(),
            loc(),
        )
        .expect("window");
        let button = bridge_button_new(
            &mut worker,
            Rect::from_coords(10, 4, 20, 6),
            "Quit".into(),
            CM_QUIT,
            false,
            loc(),
        )
        .expect("button");
        bridge_window_attach_button(&mut worker, window, button, loc()).expect("attach");
        let entry = worker
            .bridge
            .registry
            .require(button, ViewKind::Button)
            .expect("button entry");
        assert_ne!(entry.view_id, 0);
        assert!(worker.bridge.button_click_point(button).is_some());
    }
}
