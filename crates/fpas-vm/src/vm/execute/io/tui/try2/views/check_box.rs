//! Try-2 `CheckBox` construction, attach, and read-back.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::bridged_check_box::BridgedCheckBox;
use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedCheckBox, Try2Root};
use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;

/// Creates a detached check box (`CheckBox.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_check_box_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    checked: bool,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let checked_cell = TurboVisionBoolCell::new(checked);
    let check_box = Box::new(BridgedCheckBox::new(bounds, &text, checked_cell.clone()));
    Ok(worker
        .try2
        .insert_detached_check_box(check_box, bounds, checked_cell))
}

/// Returns the host-side checked state (`CheckBox.Checked`).
pub(in crate::vm::execute::io::tui::try2) fn try2_check_box_checked(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<bool, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::CheckBox)
        .map_err(|error| registry_error(error, line))?;

    if worker.try2.child_parent(handle).is_some() {
        let _ = try2_with_child_view(worker, handle, ViewKind::CheckBox, line, |view| {
            if let Some(check_box) = view.as_any_mut().downcast_mut::<BridgedCheckBox>() {
                check_box.sync_checked_from_view();
            }
            Ok(())
        });
    }

    Ok(worker
        .try2
        .check_box_cell(handle)
        .ok_or_else(|| missing_check_box_cell(handle, line))?
        .read())
}

/// Updates the checked state (`CheckBox.SetChecked`).
pub(in crate::vm::execute::io::tui::try2) fn try2_check_box_set_checked(
    worker: &mut Worker,
    handle: u32,
    checked: bool,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::CheckBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(cell) = worker.try2.check_box_cell(handle) else {
        return Err(missing_check_box_cell(handle, line));
    };
    cell.set(checked);

    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::CheckBox, line, |view| {
            if let Some(check_box) = view.as_any_mut().downcast_mut::<BridgedCheckBox>() {
                check_box.sync_from_cell();
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Attaches a detached check box to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_check_box(
    worker: &mut Worker,
    dialog_handle: u32,
    check_box_handle: u32,
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
        .require(check_box_handle, ViewKind::CheckBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedCheckBox {
        check_box,
        local_bounds,
        ..
    }) = worker.try2.take_detached_check_box(check_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("CheckBox handle {check_box_handle} is not detached"),
            "Pass a handle from `CheckBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let (view_id, hit, click) = {
        let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(dialog_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {dialog_handle} is not a Dialog"),
                "Pass a handle from `Dialog.NewModal`.",
                line,
            ));
        };
        let dialog_bounds = dialog.bounds();
        let hit = super::super::view_click::widget_screen_bounds(dialog_bounds, local_bounds);
        let click = super::super::view_click::widget_mouse_click_point(dialog_bounds, local_bounds);
        let view_id = dialog.add(check_box).as_u16();
        (view_id, hit, click)
    };

    worker
        .try2
        .register_mouse_hit_target(check_box_handle, hit, click);

    worker
        .try2
        .set_child_parent(check_box_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(check_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(check_box_handle), line))?;
    Ok(())
}

/// Attaches a detached check box to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_check_box(
    worker: &mut Worker,
    window_handle: u32,
    check_box_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.try2.is_on_desktop(window_handle) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {window_handle} is already on the desktop"),
            "Call `Window.Add` before `Desktop.Add`.",
            line,
        ));
    }

    worker
        .try2
        .registry
        .require(window_handle, ViewKind::Window)
        .map_err(|error| registry_error(error, line))?;
    worker
        .try2
        .registry
        .require(check_box_handle, ViewKind::CheckBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedCheckBox {
        check_box,
        local_bounds,
        ..
    }) = worker.try2.take_detached_check_box(check_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("CheckBox handle {check_box_handle} is not detached"),
            "Pass a handle from `CheckBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let (view_id, hit, click) = {
        let Some(Try2Root::Window(window)) = worker.try2.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        let window_bounds = window.bounds();
        let hit = super::super::view_click::widget_screen_bounds(window_bounds, local_bounds);
        let click = super::super::view_click::widget_mouse_click_point(window_bounds, local_bounds);
        let view_id = window.add(check_box).as_u16();
        (view_id, hit, click)
    };

    worker
        .try2
        .register_mouse_hit_target(check_box_handle, hit, click);

    worker
        .try2
        .set_child_parent(check_box_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(check_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(check_box_handle), line))?;
    Ok(())
}

fn missing_check_box_cell(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("CheckBox handle {handle} has no host state"),
        "Use a handle returned by `CheckBox.New` in the active try-2 session.",
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
            "Use a handle returned by `CheckBox.New` in the active session.",
        ),
        RegistryError::WrongKind {
            handle,
            expected,
            actual,
        } => (
            format!("Handle {handle} expected {:?}, got {:?}", expected, actual),
            "Pass a valid parent and child handle.",
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

    #[test]
    fn check_box_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_check_box_new(
            &mut worker,
            Rect::new(3, 2, 24, 1),
            "Enable".into(),
            true,
            loc(),
        )
        .expect("check box");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::CheckBox)
                .unwrap()
                .kind,
            ViewKind::CheckBox
        );
        assert!(try2_check_box_checked(&mut worker, handle, loc()).unwrap());
    }

    #[test]
    fn dialog_add_check_box_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let check_box = try2_check_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 1),
            "Opt in".into(),
            false,
            loc(),
        )
        .expect("check box");
        try2_dialog_attach_check_box(&mut worker, dialog, check_box, loc()).expect("attach");
        try2_check_box_set_checked(&mut worker, check_box, true, loc()).expect("set checked");
        assert!(try2_check_box_checked(&mut worker, check_box, loc()).unwrap());
    }
}
