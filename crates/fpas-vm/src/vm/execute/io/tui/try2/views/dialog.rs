//! Try-2 `Dialog` construction.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::ViewKind;
use crate::vm::execute::io::tui::try2::session::Try2Root;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;

/// Creates a modal dialog root in the try-2 session.
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_new_modal(
    worker: &mut Worker,
    bounds: Rect,
    title: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let dialog = Dialog::new_modal(bounds, &title);
    Ok(worker
        .try2
        .insert_root(Try2Root::ModalDialog(dialog), ViewKind::Dialog))
}

/// Replaces a modal dialog title (`Dialog.SetTitle`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_set_title(
    worker: &mut Worker,
    handle: u32,
    title: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Dialog)
        .map_err(|_| dialog_not_live_error(handle, line))?;

    let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(handle) else {
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

fn try2_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_CONSOLE_STATE_ERROR,
        "Try-2 TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the try-2 path.",
        line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use crate::vm::execute::io::tui::try2::registry::ViewKind;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

    #[test]
    fn new_modal_registers_dialog_root() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let bounds = Rect::new(2, 1, 30, 10);
        let handle =
            try2_dialog_new_modal(&mut worker, bounds, "Test".into(), loc()).expect("dialog");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::Dialog)
                .unwrap()
                .kind,
            ViewKind::Dialog
        );
    }
}
