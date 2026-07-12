//! Turbo Vision bridge `Dialog` construction.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::ViewKind;
use crate::vm::execute::io::tui::bridge::session::TuiRoot;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;

/// Creates a modal dialog root in the Turbo Vision session.
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_new_modal(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(bridge_session_closed_error(line));
    }

    let dialog = Dialog::new_modal(bounds, &title);
    Ok(worker
        .bridge
        .insert_root(TuiRoot::ModalDialog(dialog), ViewKind::Dialog))
}

/// Replaces a modal dialog title (`Dialog.SetTitle`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_set_title(
    worker: &mut Worker,
    handle: u32,
    title: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::Dialog)
        .map_err(|_| dialog_not_live_error(handle, line))?;

    let Some(TuiRoot::ModalDialog(dialog)) = worker.bridge.root_mut(handle) else {
        return Err(dialog_not_live_error(handle, line));
    };

    dialog.set_title(&title);
    Ok(())
}

fn dialog_not_live_error(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        format!("Dialog handle {handle} is not live"),
        "Use a handle from `Dialog.NewModal` in the active session.",
        line,
    )
}

fn bridge_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        "TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the Turbo Vision path.",
        line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use crate::vm::execute::io::tui::bridge::registry::ViewKind;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

    #[test]
    fn new_modal_registers_dialog_root() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let bounds = Rect::new(2, 1, 30, 10);
        let handle =
            bridge_dialog_new_modal(&mut worker, bounds, "Test".into(), loc()).expect("dialog");
        assert_eq!(
            worker
                .bridge
                .registry
                .require(handle, ViewKind::Dialog)
                .unwrap()
                .kind,
            ViewKind::Dialog
        );
    }
}
