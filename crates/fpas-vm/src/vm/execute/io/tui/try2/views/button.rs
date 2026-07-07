//! Try-2 `Button` construction and `Dialog.Add`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::Try2Root;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::command::CommandId;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;

/// Adds a button child to a try-2 modal dialog.
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_add_button(
    worker: &mut Worker,
    dialog_handle: u32,
    bounds: Rect,
    text: String,
    command: CommandId,
    is_default: bool,
    line: SourceLocation,
) -> Result<u32, VmError> {
    worker
        .try2
        .registry
        .require(dialog_handle, ViewKind::Dialog)
        .map_err(|error| registry_error(error, line))?;

    let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(dialog_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Handle {dialog_handle} is not a Dialog"),
            "Pass a handle from `Dialog.NewModal`.",
            line,
        ));
    };

    let button = Button::new(bounds, &text, command, is_default);
    let view_id = dialog.add(Box::new(button)).as_u16();
    Ok(worker.try2.registry.allocate(view_id, ViewKind::Button))
}

fn registry_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("Dialog handle {handle} is not live"),
            "Use a handle returned by `Dialog.NewModal` in the active session.",
        ),
        RegistryError::WrongKind {
            handle,
            expected,
            actual,
        } => (
            format!("Handle {handle} expected {:?}, got {:?}", expected, actual),
            "Pass a Dialog handle as the parent.",
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
    use crate::vm::execute::io::tui::try2::views::dialog::try2_dialog_new_modal;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::command::CM_OK;

    #[test]
    fn add_button_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog = try2_dialog_new_modal(&mut worker, Rect::new(2, 1, 30, 10), "T".into(), loc())
            .expect("dialog");
        let button = try2_dialog_add_button(
            &mut worker,
            dialog,
            Rect::new(8, 5, 18, 7),
            "OK".into(),
            CM_OK,
            true,
            loc(),
        )
        .expect("button");
        let entry = worker
            .try2
            .registry
            .require(button, ViewKind::Button)
            .expect("button entry");
        assert_ne!(entry.view_id, 0);
    }
}
