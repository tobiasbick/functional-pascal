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
    let mut file_dialog =
        FileDialog::new(bounds, &title, &wildcard, initial_dir).build();

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
    // Headless file picker still uses the try-1 queued result until upstream exposes
    // a headless-safe `FileDialog::execute` host.
    let _ = line;
    Ok(worker
        .with_tui(|tui| tui.turbo_vision.test_file_dialog_result.take())
        .unwrap_or(None)
        .map(PathBuf::from))
}
