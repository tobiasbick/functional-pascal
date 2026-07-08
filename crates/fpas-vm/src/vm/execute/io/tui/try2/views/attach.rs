//! Try-2 parent attach dispatch (`Dialog.Add` / `Window.Add` by child kind).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::button::try2_dialog_attach_button;
use super::static_text::{try2_dialog_attach_static_text, try2_window_attach_static_text};
use super::window::try2_window_attach_button;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::ViewKind;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Attaches a detached child widget to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_child(
    worker: &mut Worker,
    dialog_handle: u32,
    child_handle: u32,
    child_kind: ViewKind,
    line: SourceLocation,
) -> Result<(), VmError> {
    match child_kind {
        ViewKind::Button => try2_dialog_attach_button(worker, dialog_handle, child_handle, line),
        ViewKind::StaticText => {
            try2_dialog_attach_static_text(worker, dialog_handle, child_handle, line)
        }
        other => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Dialog.Add does not accept child kind {other:?}"),
            "Attach a `Button` or `StaticText` created with `Button.New` or `StaticText.New`.",
            line,
        )),
    }
}

/// Attaches a detached child widget to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_child(
    worker: &mut Worker,
    window_handle: u32,
    child_handle: u32,
    child_kind: ViewKind,
    line: SourceLocation,
) -> Result<(), VmError> {
    match child_kind {
        ViewKind::Button => try2_window_attach_button(worker, window_handle, child_handle, line),
        ViewKind::StaticText => {
            try2_window_attach_static_text(worker, window_handle, child_handle, line)
        }
        other => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window.Add does not accept child kind {other:?}"),
            "Attach a `Button` or `StaticText` created with `Button.New` or `StaticText.New`.",
            line,
        )),
    }
}
