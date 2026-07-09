//! Try-2 `Button` construction, attach, and `SetText`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridged_button::BridgedButton;
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedButton, Try2Root};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::command::CommandId;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::views::View;

/// Creates a detached button (`Button.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_button_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    command: CommandId,
    is_default: bool,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }
    let button = Box::new(BridgedButton::new(bounds, &text, command, is_default));
    Ok(worker
        .try2
        .insert_detached_button(button, bounds, command, is_default, text))
}

/// Replaces button label text (`Button.SetText`).
pub(in crate::vm::execute::io::tui::try2) fn try2_button_set_text(
    worker: &mut Worker,
    handle: u32,
    text: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Button)
        .map_err(|error| registry_error(error, line))?;

    worker.try2.set_button_text(handle, text.clone());

    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::Button, line, |view| {
            if let Some(button) = view.as_any_mut().downcast_mut::<BridgedButton>() {
                button.set_text_from_fpas(&text);
            }
            Ok(())
        })?;
    } else if let Some(bounds) = worker.try2.detached_button_bounds(handle) {
        let Some(state) = worker.try2.button_state(handle) else {
            return Err(missing_button_state(handle, line));
        };
        worker.try2.replace_detached_button(
            handle,
            Box::new(BridgedButton::new(
                bounds,
                &text,
                state.command,
                state.is_default,
            )),
        );
    }

    Ok(())
}

/// Attaches a detached button to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_button(
    worker: &mut Worker,
    dialog_handle: u32,
    button_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(dialog_handle, ViewKind::Dialog)
        .map_err(|error| registry_error(error, line))?;
    worker
        .try2
        .registry
        .require(button_handle, ViewKind::Button)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedButton {
        button,
        local_bounds,
    }) = worker.try2.take_detached_button(button_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Button handle {button_handle} is not detached"),
            "Pass a handle from `Button.New` that has not been added to a dialog yet.",
            line,
        ));
    };

    let (view_id, click) = {
        let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(dialog_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {dialog_handle} is not a Dialog"),
                "Pass a handle from `Dialog.NewModal`.",
                line,
            ));
        };
        let dialog_bounds = dialog.bounds();
        let view_id = dialog.add(button).as_u16();
        (view_id, button_click_point(dialog_bounds, local_bounds))
    };

    worker
        .try2
        .registry
        .set_view_id(button_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(button_handle), line))?;
    worker.try2.set_child_parent(button_handle, dialog_handle);
    worker.try2.set_button_click_point(button_handle, click);
    Ok(())
}

/// Adds a button child to a try-2 modal dialog (`Dialog.AddButton` convenience API).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_add_button(
    worker: &mut Worker,
    dialog_handle: u32,
    bounds: Rect,
    text: String,
    command: CommandId,
    is_default: bool,
    line: SourceLocation,
) -> Result<u32, VmError> {
    let button_handle = try2_button_new(worker, bounds, text, command, is_default, line)?;
    try2_dialog_attach_button(worker, dialog_handle, button_handle, line)?;
    Ok(button_handle)
}

pub(in crate::vm::execute::io::tui::try2) fn button_click_point(
    dialog_bounds: Rect,
    button_bounds: Rect,
) -> Point {
    Point::new(
        dialog_bounds.a.x + button_bounds.a.x + button_bounds.width() / 2,
        dialog_bounds.a.y + button_bounds.a.y + button_bounds.height() / 2,
    )
}

fn missing_button_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("Button handle {handle} has no try-2 host state"),
        "Use a handle returned by `Button.New` in the active session.",
        line,
    )
}

fn try2_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Try-2 TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the try-2 path.",
        line,
    )
}

fn registry_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("Handle {handle} is not live"),
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

    #[test]
    fn button_new_and_dialog_add_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog = try2_dialog_new_modal(
            &mut worker,
            Rect::from_coords(5, 3, 30, 8),
            "T".into(),
            loc(),
        )
        .expect("dialog");
        let button = try2_button_new(
            &mut worker,
            Rect::from_coords(10, 4, 20, 6),
            "OK".into(),
            CM_OK,
            true,
            loc(),
        )
        .expect("button");
        try2_dialog_attach_button(&mut worker, dialog, button, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(button, ViewKind::Button)
            .expect("button entry");
        assert_ne!(entry.view_id, 0);
        assert!(worker.try2.button_click_point(button).is_some());
    }

    #[test]
    fn set_text_updates_host_state() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_button_new(
            &mut worker,
            Rect::new(3, 2, 10, 4),
            "OLD".into(),
            CM_OK,
            false,
            loc(),
        )
        .expect("button");
        try2_button_set_text(&mut worker, handle, "NEW".into(), loc()).expect("set text");
        assert_eq!(worker.try2.button_state(handle).unwrap().text, "NEW");
    }
}
