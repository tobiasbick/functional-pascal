//! Try-2 parent attach dispatch (`Dialog.Add` / `Window.Add` by child kind).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::button::try2_dialog_attach_button;
use super::check_box::{try2_dialog_attach_check_box, try2_window_attach_check_box};
use super::input_line::{try2_dialog_attach_input_line, try2_window_attach_input_line};
use super::list_box::{try2_dialog_attach_list_box, try2_window_attach_list_box};
use super::memo::{try2_dialog_attach_memo, try2_window_attach_memo};
use super::outline::{try2_dialog_attach_outline, try2_window_attach_outline};
use super::radio_button::{try2_dialog_attach_radio_button, try2_window_attach_radio_button};
use super::static_text::{try2_dialog_attach_static_text, try2_window_attach_static_text};
use super::text_viewer::{try2_dialog_attach_text_viewer, try2_window_attach_text_viewer};
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
        ViewKind::CheckBox => {
            try2_dialog_attach_check_box(worker, dialog_handle, child_handle, line)
        }
        ViewKind::InputLine => {
            try2_dialog_attach_input_line(worker, dialog_handle, child_handle, line)
        }
        ViewKind::ListBox => try2_dialog_attach_list_box(worker, dialog_handle, child_handle, line),
        ViewKind::Outline => try2_dialog_attach_outline(worker, dialog_handle, child_handle, line),
        ViewKind::RadioButton => {
            try2_dialog_attach_radio_button(worker, dialog_handle, child_handle, line)
        }
        ViewKind::Memo => try2_dialog_attach_memo(worker, dialog_handle, child_handle, line),
        ViewKind::TextViewer => {
            try2_dialog_attach_text_viewer(worker, dialog_handle, child_handle, line)
        }
        other => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Dialog.Add does not accept child kind {other:?}"),
            "Attach a `Button`, `StaticText`, `CheckBox`, `InputLine`, `ListBox`, `Outline`, `RadioButton`, `Memo`, or `TextViewer` created with the matching `*.New` constructor.",
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
        ViewKind::CheckBox => {
            try2_window_attach_check_box(worker, window_handle, child_handle, line)
        }
        ViewKind::InputLine => {
            try2_window_attach_input_line(worker, window_handle, child_handle, line)
        }
        ViewKind::ListBox => try2_window_attach_list_box(worker, window_handle, child_handle, line),
        ViewKind::Outline => try2_window_attach_outline(worker, window_handle, child_handle, line),
        ViewKind::RadioButton => {
            try2_window_attach_radio_button(worker, window_handle, child_handle, line)
        }
        ViewKind::Memo => try2_window_attach_memo(worker, window_handle, child_handle, line),
        ViewKind::TextViewer => {
            try2_window_attach_text_viewer(worker, window_handle, child_handle, line)
        }
        other => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window.Add does not accept child kind {other:?}"),
            "Attach a `Button`, `StaticText`, `CheckBox`, `InputLine`, `ListBox`, `Outline`, `RadioButton`, `Memo`, or `TextViewer` created with the matching `*.New` constructor.",
            line,
        )),
    }
}
