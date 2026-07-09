//! Try-2 `RadioButton` construction, attach, and selection read-back.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridged_radio_button::BridgedRadioButton;
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedRadioButton, Try2Root};
use crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;

/// Creates a detached radio button (`RadioButton.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_radio_button_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    group_id: u16,
    selected: bool,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let selected_cell = TurboVisionBoolCell::new(selected);
    let handle = worker.try2.insert_radio_button_state(
        bounds,
        text.clone(),
        group_id,
        selected_cell.clone(),
    );
    if selected {
        worker
            .try2
            .deselect_radio_group_except(group_id, Some(handle));
        selected_cell.set(true);
    }
    let radio_button = build_bridged_radio_button(
        bounds,
        &text,
        group_id,
        selected_cell.clone(),
        worker.try2.radio_group_cells(group_id),
    );
    worker
        .try2
        .insert_detached_radio_button(handle, radio_button, bounds);
    try2_refresh_radio_group_bridges(worker, group_id, line)?;
    Ok(handle)
}

/// Returns the host-side selected state (`RadioButton.Selected`).
pub(in crate::vm::execute::io::tui::try2) fn try2_radio_button_selected(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<bool, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::RadioButton)
        .map_err(|error| registry_error(error, line))?;

    if worker.try2.child_parent(handle).is_some() {
        let _ = try2_with_child_view(worker, handle, ViewKind::RadioButton, line, |view| {
            if let Some(radio) = view.as_any_mut().downcast_mut::<BridgedRadioButton>() {
                radio.sync_selected_from_view();
            }
            Ok(())
        });
    }

    Ok(worker
        .try2
        .radio_button_selected_cell(handle)
        .ok_or_else(|| missing_radio_button_state(handle, line))?
        .read())
}

/// Updates the selected state (`RadioButton.SetSelected`).
pub(in crate::vm::execute::io::tui::try2) fn try2_radio_button_set_selected(
    worker: &mut Worker,
    handle: u32,
    selected: bool,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::RadioButton)
        .map_err(|error| registry_error(error, line))?;

    let Some(state) = worker.try2.radio_button_state(handle) else {
        return Err(missing_radio_button_state(handle, line));
    };
    let group_id = state.group_id;

    if selected {
        worker
            .try2
            .deselect_radio_group_except(group_id, Some(handle));
    }
    let Some(cell) = worker.try2.radio_button_selected_cell(handle) else {
        return Err(missing_radio_button_state(handle, line));
    };
    cell.set(selected);

    try2_refresh_radio_group_bridges(worker, group_id, line)?;
    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::RadioButton, line, |view| {
            if let Some(radio) = view.as_any_mut().downcast_mut::<BridgedRadioButton>() {
                radio.sync_from_cell();
            }
            Ok(())
        })?;
    }
    Ok(())
}

/// Attaches a detached radio button to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_radio_button(
    worker: &mut Worker,
    dialog_handle: u32,
    radio_button_handle: u32,
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
        .require(radio_button_handle, ViewKind::RadioButton)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedRadioButton {
        radio_button,
        local_bounds,
        ..
    }) = worker.try2.take_detached_radio_button(radio_button_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("RadioButton handle {radio_button_handle} is not detached"),
            "Pass a handle from `RadioButton.New` that has not been added to a parent yet.",
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
        let view_id = dialog.add(radio_button).as_u16();
        (view_id, hit, click)
    };

    worker.try2.register_mouse_hit_target(hit, click);

    worker
        .try2
        .set_child_parent(radio_button_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(radio_button_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(radio_button_handle), line))?;
    Ok(())
}

/// Attaches a detached radio button to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_radio_button(
    worker: &mut Worker,
    window_handle: u32,
    radio_button_handle: u32,
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
        .require(radio_button_handle, ViewKind::RadioButton)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedRadioButton {
        radio_button,
        local_bounds,
        ..
    }) = worker.try2.take_detached_radio_button(radio_button_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("RadioButton handle {radio_button_handle} is not detached"),
            "Pass a handle from `RadioButton.New` that has not been added to a parent yet.",
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
        let view_id = window.add(radio_button).as_u16();
        (view_id, hit, click)
    };

    worker.try2.register_mouse_hit_target(hit, click);

    worker
        .try2
        .set_child_parent(radio_button_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(radio_button_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(radio_button_handle), line))?;
    Ok(())
}

fn build_bridged_radio_button(
    bounds: Rect,
    text: &str,
    group_id: u16,
    selected_cell: TurboVisionBoolCell,
    group_cells: Vec<TurboVisionBoolCell>,
) -> Box<dyn View> {
    let tree_dirty = TurboVisionBoolCell::new(false);
    Box::new(BridgedRadioButton::new(
        bounds,
        text,
        group_id,
        selected_cell,
        group_cells,
        tree_dirty,
    ))
}

fn try2_refresh_radio_group_bridges(
    worker: &mut Worker,
    group_id: u16,
    line: SourceLocation,
) -> Result<(), VmError> {
    let members = worker.try2.radio_group_member_handles(group_id);
    let group_cells = worker.try2.radio_group_cells(group_id);
    for member in members {
        if worker.try2.child_parent(member).is_some() {
            try2_with_child_view(worker, member, ViewKind::RadioButton, line, |view| {
                if let Some(radio) = view.as_any_mut().downcast_mut::<BridgedRadioButton>() {
                    radio.update_group_cells(group_cells.clone());
                    radio.sync_from_cell();
                }
                Ok(())
            })?;
            continue;
        }

        let Some(state) = worker.try2.radio_button_state(member).cloned() else {
            continue;
        };
        let radio_button = build_bridged_radio_button(
            state.bounds,
            &state.text,
            state.group_id,
            state.selected_cell.clone(),
            group_cells.clone(),
        );
        worker
            .try2
            .replace_detached_radio_button(member, radio_button);
    }
    Ok(())
}

fn missing_radio_button_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("RadioButton handle {handle} has no host state"),
        "Use a handle returned by `RadioButton.New` in the active try-2 session.",
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
            "Use a handle returned by `RadioButton.New` in the active session.",
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
    fn radio_button_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_radio_button_new(
            &mut worker,
            Rect::new(3, 2, 20, 1),
            "first".into(),
            7,
            true,
            loc(),
        )
        .expect("radio button");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::RadioButton)
                .unwrap()
                .kind,
            ViewKind::RadioButton
        );
        assert!(try2_radio_button_selected(&mut worker, handle, loc()).unwrap());
    }

    #[test]
    fn radio_group_excludes_other_members_when_selected() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let first = try2_radio_button_new(
            &mut worker,
            Rect::new(3, 2, 20, 1),
            "first".into(),
            9,
            true,
            loc(),
        )
        .expect("first");
        let second = try2_radio_button_new(
            &mut worker,
            Rect::new(3, 3, 20, 1),
            "second".into(),
            9,
            true,
            loc(),
        )
        .expect("second");
        assert!(!try2_radio_button_selected(&mut worker, first, loc()).unwrap());
        assert!(try2_radio_button_selected(&mut worker, second, loc()).unwrap());
    }

    #[test]
    fn dialog_add_radio_button_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let radio = try2_radio_button_new(
            &mut worker,
            Rect::new(3, 2, 20, 1),
            "pick me".into(),
            3,
            false,
            loc(),
        )
        .expect("radio");
        try2_dialog_attach_radio_button(&mut worker, dialog, radio, loc()).expect("attach");
        try2_radio_button_set_selected(&mut worker, radio, true, loc()).expect("select");
        assert!(try2_radio_button_selected(&mut worker, radio, loc()).unwrap());
    }
}
