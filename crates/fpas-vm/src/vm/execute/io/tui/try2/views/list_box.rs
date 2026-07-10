//! Try-2 `ListBox` construction, attach, selection read-back, and `SetItems`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridged_list_box::BridgedListBox;
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedListBox, Try2Root};
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

/// Creates a detached list box (`ListBox.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_list_box_new(
    worker: &mut Worker,
    bounds: Rect,
    items: Vec<String>,
    command: u16,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let selection = initial_list_selection(&items);
    let selection_cell = TurboVisionListSelectionCell::new(selection);
    let list_box = Box::new(BridgedListBox::new(
        bounds,
        items.clone(),
        command,
        selection_cell.clone(),
    ));
    Ok(worker
        .try2
        .insert_detached_list_box(list_box, bounds, items, command, selection_cell))
}

/// Returns the selected item index, or `-1` when empty (`ListBox.Selection`).
pub(in crate::vm::execute::io::tui::try2) fn try2_list_box_selection(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<i64, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    if worker.try2.child_parent(handle).is_some() {
        let _ = try2_with_child_view(worker, handle, ViewKind::ListBox, line, |view| {
            if let Some(list_box) = view.as_any_mut().downcast_mut::<BridgedListBox>() {
                list_box.sync_selection_from_view();
            }
            Ok(())
        });
    }

    Ok(worker
        .try2
        .list_box_selection_cell(handle)
        .ok_or_else(|| missing_list_box_state(handle, line))?
        .read()
        .map(|selection| selection as i64)
        .unwrap_or(-1))
}

/// Replaces list items (`ListBox.SetItems`).
pub(in crate::vm::execute::io::tui::try2) fn try2_list_box_set_items(
    worker: &mut Worker,
    handle: u32,
    items: Vec<String>,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let selection = initial_list_selection(&items);
    let Some(state) = worker.try2.list_box_state_mut(handle) else {
        return Err(missing_list_box_state(handle, line));
    };
    state.items = items.clone();
    state.selection_cell.set(selection);

    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::ListBox, line, |view| {
            if let Some(list_box) = view.as_any_mut().downcast_mut::<BridgedListBox>() {
                list_box.set_items_from_fpas(items, selection);
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Attaches a detached list box to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_list_box(
    worker: &mut Worker,
    dialog_handle: u32,
    list_box_handle: u32,
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
        .require(list_box_handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedListBox { list_box, .. }) =
        worker.try2.take_detached_list_box(list_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("ListBox handle {list_box_handle} is not detached"),
            "Pass a handle from `ListBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let view_id = {
        let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(dialog_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {dialog_handle} is not a Dialog"),
                "Pass a handle from `Dialog.NewModal`.",
                line,
            ));
        };
        dialog.add(list_box).as_u16()
    };

    worker.try2.set_child_parent(list_box_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(list_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(list_box_handle), line))?;
    Ok(())
}

/// Attaches a detached list box to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_list_box(
    worker: &mut Worker,
    window_handle: u32,
    list_box_handle: u32,
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
        .require(list_box_handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedListBox { list_box, .. }) =
        worker.try2.take_detached_list_box(list_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("ListBox handle {list_box_handle} is not detached"),
            "Pass a handle from `ListBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let view_id = {
        let Some(Try2Root::Window(window)) = worker.try2.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        window.add(list_box).as_u16()
    };

    worker.try2.set_child_parent(list_box_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(list_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(list_box_handle), line))?;
    Ok(())
}

fn missing_list_box_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("ListBox handle {handle} has no host state"),
        "Use a handle returned by `ListBox.New` in the active try-2 session.",
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
            "Use a handle returned by `ListBox.New` in the active session.",
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

fn initial_list_selection(items: &[String]) -> Option<usize> {
    if items.is_empty() { None } else { Some(0) }
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
    fn list_box_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into(), "beta".into()],
            100,
            loc(),
        )
        .expect("list box");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::ListBox)
                .unwrap()
                .kind,
            ViewKind::ListBox
        );
        assert_eq!(
            try2_list_box_selection(&mut worker, handle, loc()).unwrap(),
            0
        );
    }

    #[test]
    fn set_items_updates_selection() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into()],
            100,
            loc(),
        )
        .expect("list box");
        try2_list_box_set_items(&mut worker, handle, vec![], loc()).expect("clear items");
        assert_eq!(
            try2_list_box_selection(&mut worker, handle, loc()).unwrap(),
            -1
        );
        try2_list_box_set_items(&mut worker, handle, vec!["one".into()], loc())
            .expect("set one item");
        assert_eq!(
            try2_list_box_selection(&mut worker, handle, loc()).unwrap(),
            0
        );
    }

    #[test]
    fn dialog_add_list_box_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let list_box = try2_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into(), "beta".into()],
            100,
            loc(),
        )
        .expect("list box");
        try2_dialog_attach_list_box(&mut worker, dialog, list_box, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(list_box, ViewKind::ListBox)
            .expect("list box entry");
        assert_ne!(entry.view_id, 0);
    }
}
