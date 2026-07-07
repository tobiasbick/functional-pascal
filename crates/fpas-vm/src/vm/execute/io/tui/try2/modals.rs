//! Try-2 modal execution (`Application.ExecView`).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::app::try2_ensure_live_app;
use super::headless::try2_headless_exec_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::ViewKind;
use crate::vm::execute::io::tui::try2::session::Try2Root;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::command::CommandId;

/// Runs a modal dialog on the live or headless turbo-vision application.
pub(in crate::vm::execute::io::tui::try2) fn try2_exec_view(
    worker: &mut Worker,
    dialog_handle: u32,
    line: SourceLocation,
) -> Result<CommandId, VmError> {
    if worker.current_task_id != 0 {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.ExecView must run on the main task",
            "Call `Application.ExecView` from the main program, not from a `go` task.",
            line,
        ));
    }

    worker
        .try2
        .registry
        .require(dialog_handle, ViewKind::Dialog)
        .map_err(|_| dialog_not_live_error(dialog_handle, line))?;

    let Some(Try2Root::ModalDialog(dialog)) = worker.try2.take_root(dialog_handle) else {
        return Err(dialog_not_live_error(dialog_handle, line));
    };

    if worker.with_tui(|tui| tui.session.is_headless()) {
        return try2_headless_exec_view(worker, dialog, line);
    }

    try2_ensure_live_app(worker, line)?;
    let Some(app) = worker.live_turbo_vision_app.as_mut() else {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Turbo Vision live session is not initialized",
            "Call `Application.Open()` before `Application.ExecView`.",
            line,
        ));
    };
    Ok(app.exec_view(dialog))
}

fn dialog_not_live_error(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!("Dialog handle {handle} is not live"),
        "Use a handle from `Dialog.NewModal` in the active session.",
        line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use crate::vm::execute::io::tui::try2::headless::try2_ensure_headless_app;
    use crate::vm::execute::io::tui::try2::views::{
        try2_dialog_add_button, try2_dialog_new_modal,
    };
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_OK;
    use turbo_vision::core::event::KB_ENTER;
    use turbo_vision::core::geometry::Rect;

    fn headless_try2_worker(width: u16, height: u16) -> Worker {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.with_console(|console| console.resize(width, height));
        {
            let mut tui = worker.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            worker
                .with_console(|console| {
                    tui.session
                        .open_for_test(console, loc())
                        .expect("open_for_test");
                });
        }
        worker.open_try2_session();
        worker
    }

    #[test]
    fn headless_exec_view_default_ok_button_returns_cm_ok() {
        let mut worker = headless_try2_worker(40, 14);
        let dialog = try2_dialog_new_modal(
            &mut worker,
            Rect::new(5, 3, 30, 8),
            "Test".into(),
            loc(),
        )
        .expect("dialog");
        try2_dialog_add_button(
            &mut worker,
            dialog,
            Rect::new(10, 4, 20, 6),
            "OK".into(),
            CM_OK,
            true,
            loc(),
        )
        .expect("button");

        try2_ensure_headless_app(&mut worker, loc()).expect("headless app");
        worker
            .headless_tv_app
            .as_ref()
            .expect("headless app")
            .push_keyboard(KB_ENTER);

        let command = try2_exec_view(&mut worker, dialog, loc()).expect("exec_view");
        assert_eq!(command, CM_OK);
    }
}
