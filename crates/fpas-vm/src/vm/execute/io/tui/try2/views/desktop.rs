//! Try-2 `Desktop.Add` — attach modeless windows to the live or headless desktop.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::app::try2_ensure_live_app;
use super::super::headless::try2_ensure_headless_app;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::Try2Root;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::View;
/// Adds a try-2 window to the upstream desktop (`Desktop.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_desktop_add(
    worker: &mut Worker,
    window_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    if !worker.try2.is_open() {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Try-2 TUI session is not open",
            "Call `Application.Open` before `Desktop.Add`.",
            line,
        ));
    }

    if worker.try2.is_on_desktop(window_handle) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {window_handle} is already on the desktop"),
            "Each window may only be added once per session.",
            line,
        ));
    }

    let kind = worker
        .try2
        .registry
        .get(window_handle)
        .ok_or_else(|| registry_error(RegistryError::UnknownHandle(window_handle), line))?
        .kind;
    if kind != ViewKind::Window && kind != ViewKind::EditorWindow {
        return Err(registry_error(
            RegistryError::WrongKind {
                handle: window_handle,
                expected: ViewKind::Window,
                actual: kind,
            },
            line,
        ));
    }
    let Some(root) = worker.try2.take_root(window_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Handle {window_handle} is not a Window"),
            "Pass a handle from `Window.New`.",
            line,
        ));
    };
    let window: Box<dyn View> = match root {
        Try2Root::Window(window) => window,
        Try2Root::EditorWindow(window) => window,
        Try2Root::ModalDialog(_) => unreachable!("validated desktop root"),
    };

    let view_id = if worker.with_tui(|tui| tui.session.is_headless()) {
        try2_ensure_headless_app(worker, line)?;
        let Some(app) = worker.headless_tv_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before `Desktop.Add`.",
                line,
            ));
        };
        app.desktop_mut().add(window).as_u16()
    } else {
        try2_ensure_live_app(worker, line)?;
        let Some(app) = worker.live_turbo_vision_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Turbo Vision live session is not initialized",
                "Call `Application.Open` before `Desktop.Add`.",
                line,
            ));
        };
        app.desktop.add(window).as_u16()
    };

    worker.try2.mark_desktop_window(window_handle);
    worker
        .try2
        .registry
        .set_view_id(window_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(window_handle), line))?;
    Ok(())
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
            "Pass a Window handle to `Desktop.Add`.",
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
    use crate::vm::execute::io::tui::try2::views::window::{
        try2_window_attach_button, try2_window_new,
    };
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_QUIT;
    use turbo_vision::core::geometry::Rect;

    fn headless_try2_worker(width: u16, height: u16) -> Worker {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.with_console(|console| console.resize(width, height));
        {
            let mut tui = worker.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            worker.with_console(|console| {
                tui.session
                    .open_for_test(console, loc())
                    .expect("open_for_test");
            });
        }
        worker.open_try2_session();
        worker
    }

    #[test]
    fn desktop_add_registers_upstream_view_id() {
        let mut worker = headless_try2_worker(60, 20);
        let window = try2_window_new(&mut worker, Rect::new(5, 3, 30, 10), "Test".into(), loc())
            .expect("window");
        try2_desktop_add(&mut worker, window, loc()).expect("desktop add");
        let entry = worker
            .try2
            .registry
            .require(window, ViewKind::Window)
            .expect("window entry");
        assert_ne!(entry.view_id, 0);
        assert!(worker.try2.is_on_desktop(window));
    }

    #[test]
    fn window_with_quit_button_on_desktop() {
        let mut worker = headless_try2_worker(60, 20);
        let window = try2_window_new(&mut worker, Rect::new(5, 3, 30, 10), "Test".into(), loc())
            .expect("window");
        let button = try2_button_new(
            &mut worker,
            Rect::new(10, 4, 20, 6),
            "Quit".into(),
            CM_QUIT,
            false,
            loc(),
        )
        .expect("button");
        try2_window_attach_button(&mut worker, window, button, loc()).expect("attach");
        try2_desktop_add(&mut worker, window, loc()).expect("desktop add");
        assert!(worker.try2.button_click_point(button).is_some());
    }

    #[test]
    fn desktop_button_click_produces_quit_command_in_run_step() {
        let mut worker = headless_try2_worker(60, 20);
        let window = try2_window_new(
            &mut worker,
            Rect::from_coords(5, 3, 35, 13),
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
        try2_desktop_add(&mut worker, window, loc()).expect("desktop add");

        let point = worker.try2.button_click_point(button).expect("click point");
        try2_ensure_headless_app(&mut worker, loc()).expect("headless app");
        let app = worker.headless_tv_app.as_mut().expect("app");
        app.push_mouse_down(point.x, point.y);
        app.push_mouse_up(point.x, point.y);

        let mut command = None;
        for _ in 0..16 {
            if let Ok(Some(cmd)) = app.run_step() {
                command = Some(cmd);
                break;
            }
        }
        assert_eq!(command, Some(CM_QUIT));
    }
}
