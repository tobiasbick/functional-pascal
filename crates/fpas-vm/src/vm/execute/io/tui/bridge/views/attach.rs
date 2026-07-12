//! Turbo Vision bridge parent attach dispatch (`Dialog.Add` / `Window.Add` by child kind).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::button::bridge_dialog_attach_button;
use super::check_box::{bridge_dialog_attach_check_box, bridge_window_attach_check_box};
use super::input_line::{bridge_dialog_attach_input_line, bridge_window_attach_input_line};
use super::list_box::{bridge_dialog_attach_list_box, bridge_window_attach_list_box};
use super::memo::{bridge_dialog_attach_memo, bridge_window_attach_memo};
use super::outline::{bridge_dialog_attach_outline, bridge_window_attach_outline};
use super::radio_button::{bridge_dialog_attach_radio_button, bridge_window_attach_radio_button};
use super::static_text::{bridge_dialog_attach_static_text, bridge_window_attach_static_text};
use super::text_viewer::{bridge_dialog_attach_text_viewer, bridge_window_attach_text_viewer};
use super::window::bridge_window_attach_button;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::ViewKind;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Attaches a detached child widget to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_attach_child(
    worker: &mut Worker,
    dialog_handle: u32,
    child_handle: u32,
    child_kind: ViewKind,
    line: SourceLocation,
) -> Result<(), VmError> {
    match child_kind {
        ViewKind::Button => bridge_dialog_attach_button(worker, dialog_handle, child_handle, line),
        ViewKind::StaticText => {
            bridge_dialog_attach_static_text(worker, dialog_handle, child_handle, line)
        }
        ViewKind::CheckBox => {
            bridge_dialog_attach_check_box(worker, dialog_handle, child_handle, line)
        }
        ViewKind::InputLine => {
            bridge_dialog_attach_input_line(worker, dialog_handle, child_handle, line)
        }
        ViewKind::ListBox => {
            bridge_dialog_attach_list_box(worker, dialog_handle, child_handle, line)
        }
        ViewKind::Outline => {
            bridge_dialog_attach_outline(worker, dialog_handle, child_handle, line)
        }
        ViewKind::RadioButton => {
            bridge_dialog_attach_radio_button(worker, dialog_handle, child_handle, line)
        }
        ViewKind::Memo => bridge_dialog_attach_memo(worker, dialog_handle, child_handle, line),
        ViewKind::TextViewer => {
            bridge_dialog_attach_text_viewer(worker, dialog_handle, child_handle, line)
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
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_attach_child(
    worker: &mut Worker,
    window_handle: u32,
    child_handle: u32,
    child_kind: ViewKind,
    line: SourceLocation,
) -> Result<(), VmError> {
    match child_kind {
        ViewKind::Button => bridge_window_attach_button(worker, window_handle, child_handle, line),
        ViewKind::StaticText => {
            bridge_window_attach_static_text(worker, window_handle, child_handle, line)
        }
        ViewKind::CheckBox => {
            bridge_window_attach_check_box(worker, window_handle, child_handle, line)
        }
        ViewKind::InputLine => {
            bridge_window_attach_input_line(worker, window_handle, child_handle, line)
        }
        ViewKind::ListBox => {
            bridge_window_attach_list_box(worker, window_handle, child_handle, line)
        }
        ViewKind::Outline => {
            bridge_window_attach_outline(worker, window_handle, child_handle, line)
        }
        ViewKind::RadioButton => {
            bridge_window_attach_radio_button(worker, window_handle, child_handle, line)
        }
        ViewKind::Memo => bridge_window_attach_memo(worker, window_handle, child_handle, line),
        ViewKind::TextViewer => {
            bridge_window_attach_text_viewer(worker, window_handle, child_handle, line)
        }
        other => Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window.Add does not accept child kind {other:?}"),
            "Attach a `Button`, `StaticText`, `CheckBox`, `InputLine`, `ListBox`, `Outline`, `RadioButton`, `Memo`, or `TextViewer` created with the matching `*.New` constructor.",
            line,
        )),
    }
}
