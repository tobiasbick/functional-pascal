//! Try-2 `Application.RunFileDialog` on the live or headless turbo-vision session.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::app::try2_ensure_live_app;
use super::chrome::try2_sync_chrome_to_app;
use super::headless::try2_ensure_headless_app;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::path::PathBuf;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::file_dialog::FileDialog;

/// Runs a modal file dialog when the try-2 session is active.
pub(in crate::vm::execute::io::tui) fn try2_run_file_dialog(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    wildcard: String,
    start_path: Option<String>,
    line: SourceLocation,
) -> Result<Option<String>, VmError> {
    if worker.current_task_id != 0 {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.RunFileDialog(App, ...) must run on the main task",
            "Call `Application.RunFileDialog` from the main program, not from a `go` task.",
            line,
        ));
    }

    try2_sync_chrome_to_app(worker, line)?;

    let initial_dir = start_path
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    let mut file_dialog = FileDialog::new(bounds, &title, &wildcard, initial_dir).build();

    let selected = if worker.with_tui(|tui| tui.session.is_headless()) {
        try2_headless_run_file_dialog(worker, line)?
    } else {
        try2_ensure_live_app(worker, line)?;
        let Some(app) = worker.live_turbo_vision_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Turbo Vision live session is not initialized",
                "Call `Application.Open` before `Application.RunFileDialog`.",
                line,
            ));
        };
        file_dialog.execute(app)
    };

    Ok(selected.map(|path| path.to_string_lossy().into_owned()))
}

fn try2_headless_run_file_dialog(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<Option<PathBuf>, VmError> {
    try2_ensure_headless_app(worker, line)?;
    let _ = line;
    Ok(worker
        .try2
        .take_file_dialog_result()
        .unwrap_or(None)
        .map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

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
    fn headless_file_dialog_uses_try2_queue() {
        let mut worker = headless_try2_worker(80, 25);
        worker
            .try2
            .set_file_dialog_result(Some("picked.txt".into()));

        let selected = try2_run_file_dialog(
            &mut worker,
            Rect::new(10, 5, 50, 14),
            "Open File".into(),
            "*".into(),
            None,
            loc(),
        )
        .expect("file dialog");

        assert_eq!(selected, Some("picked.txt".into()));
    }
}
