//! Try-2 `Window` construction and `Window.Add`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::button::button_click_point;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::Try2Root;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::window::Window;

/// Creates a modeless window root in the try-2 session (`Window.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_new(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let window = Window::new(bounds, &title);
    Ok(worker
        .try2
        .insert_root(Try2Root::Window(Box::new(window)), ViewKind::Window))
}

/// Attaches a detached button to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_button(
    worker: &mut Worker,
    window_handle: u32,
    button_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.try2.is_on_desktop(window_handle) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {window_handle} is already on the desktop"),
            "Call `Window.Add` before `Desktop.Add`.",
            line,
        ));
    }

    worker
        .try2
        .registry
        .require(window_handle, ViewKind::Window)
        .map_err(|error| registry_error(error, line))?;
    worker
        .try2
        .registry
        .require(button_handle, ViewKind::Button)
        .map_err(|error| registry_error(error, line))?;

    let Some(detached) = worker.try2.take_detached_button(button_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Button handle {button_handle} is not detached"),
            "Pass a handle from `Button.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let (view_id, click) = {
        let Some(Try2Root::Window(window)) = worker.try2.root_mut(window_handle) else {
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
        .try2
        .registry
        .set_view_id(button_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(button_handle), line))?;
    worker.try2.set_button_click_point(button_handle, click);
    Ok(())
}

fn try2_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Try-2 TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the try-2 path.",
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
    use crate::vm::execute::io::tui::try2::registry::ViewKind;
    use crate::vm::execute::io::tui::try2::views::button::try2_button_new;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_QUIT;

    #[test]
    fn new_registers_window_root() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let bounds = Rect::new(5, 3, 30, 10);
        let handle = try2_window_new(&mut worker, bounds, "Test".into(), loc()).expect("window");
        assert_eq!(
            worker
                .try2
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
        worker.try2.open();
        let window = try2_window_new(
            &mut worker,
            Rect::from_coords(5, 3, 30, 10),
            "Test".into(),
            loc(),
        )
        .expect("window");
        let button = try2_button_new(
            &mut worker,
            Rect::from_coords(10, 4, 20, 6),
            "Quit".into(),
            CM_QUIT,
            false,
            loc(),
        )
        .expect("button");
        try2_window_attach_button(&mut worker, window, button, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(button, ViewKind::Button)
            .expect("button entry");
        assert_ne!(entry.view_id, 0);
        assert!(worker.try2.button_click_point(button).is_some());
    }
}
