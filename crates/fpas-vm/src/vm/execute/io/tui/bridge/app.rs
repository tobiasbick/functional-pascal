//! Turbo Vision bridge live `Application` session.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::app::Application as TurboVisionApplication;

/// Creates or reuses the live turbo-vision application for Turbo Vision (empty desktop).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_ensure_live_app(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.with_tui(|tui| tui.session.is_headless()) {
        return Ok(());
    }
    if worker.turbo_vision_live_app_active() {
        return Ok(());
    }

    let app = TurboVisionApplication::new().map_err(|error| {
        runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            format!("Turbo Vision terminal initialization failed: {error}"),
            "Run the program from an interactive terminal or use `Application.OpenForTest` in automated tests.",
            line,
        )
    })?;
    worker.live_turbo_vision_app = Some(app);
    Ok(())
}
