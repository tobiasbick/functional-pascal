//! Resolve try-2 child widgets inside owned dialog or window roots.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

use super::registry::{RegistryError, ViewKind};
use super::session::Try2Root;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::views::{View, ViewId};

/// Mutably accesses an attached child view still owned by a try-2 root.
pub(in crate::vm::execute::io::tui::try2) fn try2_with_child_view<F, R>(
    worker: &mut Worker,
    child_handle: u32,
    expected: ViewKind,
    line: SourceLocation,
    f: F,
) -> Result<R, VmError>
where
    F: FnOnce(&mut dyn View) -> Result<R, VmError>,
{
    let view_id = worker
        .try2
        .registry
        .require(child_handle, expected)
        .map_err(|error| registry_error(error, line))?
        .view_id;
    let Some(parent_handle) = worker.try2.child_parent(child_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Handle {child_handle} is not attached to a parent"),
            "Call `Dialog.Add` or `Window.Add` before reading or mutating the widget.",
            line,
        ));
    };

    let Some(root) = worker.try2.root_mut(parent_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Parent handle {parent_handle} is not an owned try-2 root"),
            "Read or mutate the widget before `Desktop.Add` or `Application.ExecView` consumes the parent.",
            line,
        ));
    };

    let view_id = ViewId::from_u16(view_id);
    let child = match root {
        Try2Root::ModalDialog(dialog) => dialog.child_by_id_mut(view_id),
        Try2Root::Window(window) => window.child_by_id_mut(view_id),
    };

    let Some(child) = child else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Child view id {} is not live under parent {parent_handle}",
                view_id.as_u16()
            ),
            "Use a handle returned by the try-2 widget constructor in the active session.",
            line,
        ));
    };

    f(child)
}

fn registry_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("Handle {handle} is not live"),
            "Use a handle returned by the try-2 widget constructor in the active session.",
        ),
        RegistryError::WrongKind {
            handle,
            expected,
            actual,
        } => (
            format!("Handle {handle} expected {:?}, got {:?}", expected, actual),
            "Pass a handle of the expected widget type.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}
