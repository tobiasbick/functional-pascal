//! Turbo Vision bridge `EditorWindow` construction backed by upstream `EditWindow`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::ViewKind;
use crate::vm::execute::io::tui::bridge::session::TuiRoot;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::edit_window::EditWindow;

/// Creates an editor window with upstream scrollbars and position indicator.
pub(in crate::vm::execute::io::tui::bridge) fn bridge_editor_window_new(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "TUI session is not open",
            "Call `Application.New` before creating an editor window.",
            line,
        ));
    }
    Ok(worker.bridge.insert_root(
        TuiRoot::EditorWindow(Box::new(EditWindow::new(bounds, &title))),
        ViewKind::EditorWindow,
    ))
}
