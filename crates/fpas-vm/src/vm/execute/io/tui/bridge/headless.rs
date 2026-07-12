//! Turbo Vision bridge headless modal execution via [`HeadlessTvApp`].
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::command::CommandId;
use turbo_vision::views::View;

/// Ensures a headless turbo-vision session sized to the FPAS console.
pub(in crate::vm::execute::io::tui::bridge) fn bridge_ensure_headless_app(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    let width = worker.with_console(|console| console.screen_width() as u16);
    let height = worker.with_console(|console| console.screen_height() as u16);
    worker
        .turbo_vision_ensure_headless_app(width, height)
        .map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Headless Turbo Vision initialization failed: {error}"),
                "Call `Application.OpenForTest` before Turbo Vision headless modals.",
                line,
            )
        })
}

/// Runs a modal on the headless desktop and exports the painted buffer.
pub(in crate::vm::execute::io::tui::bridge) fn bridge_headless_exec_view(
    worker: &mut Worker,
    dialog: Box<dyn View>,
    line: SourceLocation,
) -> Result<CommandId, VmError> {
    bridge_ensure_headless_app(worker, line)?;

    let mut app_slot = worker.headless_tv_app.take().ok_or_else(|| {
        runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Headless Turbo Vision session is not initialized",
            "Call `Application.OpenForTest` before `Application.ExecView`.",
            line,
        )
    })?;

    let command = app_slot.exec_modal_view(dialog);
    worker.headless_tv_app = Some(app_slot);
    worker.turbo_vision_export_headless_to_console();
    Ok(command)
}
