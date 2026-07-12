//! Turbo Vision bridge `ListBox` construction, attach, selection read-back, and `SetItems`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::super::view_lookup::bridge_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::bridge::session::{DetachedListBox, TuiRoot};
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::listbox::ListBox;

/// Creates a detached list box (`ListBox.New`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_list_box_new(
    worker: &mut Worker,
    bounds: Rect,
    items: Vec<String>,
    command: u16,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(bridge_session_closed_error(line));
    }

    let selection = initial_list_selection(&items);
    let selection_cell = TurboVisionListSelectionCell::new(selection);
    let list_box = Box::new(list_box_with_items(
        bounds,
        items.clone(),
        command,
        selection,
    ));
    Ok(worker
        .bridge
        .insert_detached_list_box(list_box, items, selection_cell))
}

/// Returns the selected item index, or `-1` when empty (`ListBox.Selection`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_list_box_selection(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<i64, VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    if worker.bridge.child_parent(handle).is_some() {
        let selection = bridge_with_child_view(worker, handle, ViewKind::ListBox, line, |view| {
            let Some(list_box) = view.as_any_mut().downcast_mut::<ListBox>() else {
                return Err(missing_list_box_view(handle, line));
            };
            Ok(list_box.get_selection())
        })?;
        let Some(cell) = worker.bridge.list_box_selection_cell(handle) else {
            return Err(missing_list_box_state(handle, line));
        };
        cell.set(selection);
    }

    Ok(worker
        .bridge
        .list_box_selection_cell(handle)
        .ok_or_else(|| missing_list_box_state(handle, line))?
        .read()
        .map(|selection| selection as i64)
        .unwrap_or(-1))
}

/// Replaces list items (`ListBox.SetItems`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_list_box_set_items(
    worker: &mut Worker,
    handle: u32,
    items: Vec<String>,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let selection = initial_list_selection(&items);
    let Some(state) = worker.bridge.list_box_state_mut(handle) else {
        return Err(missing_list_box_state(handle, line));
    };
    state.items = items.clone();
    state.selection_cell.set(selection);

    if worker.bridge.child_parent(handle).is_some() {
        bridge_with_child_view(worker, handle, ViewKind::ListBox, line, |view| {
            let Some(list_box) = view.as_any_mut().downcast_mut::<ListBox>() else {
                return Err(missing_list_box_view(handle, line));
            };
            list_box.set_items(items);
            if let Some(selection) = selection {
                list_box.set_selection(selection);
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Attaches a detached list box to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_attach_list_box(
    worker: &mut Worker,
    dialog_handle: u32,
    list_box_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(dialog_handle, ViewKind::Dialog)
        .map_err(|error| registry_error(error, line))?;
    worker
        .bridge
        .registry
        .require(list_box_handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedListBox { list_box, .. }) =
        worker.bridge.take_detached_list_box(list_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("ListBox handle {list_box_handle} is not detached"),
            "Pass a handle from `ListBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let view_id = {
        let Some(TuiRoot::ModalDialog(dialog)) = worker.bridge.root_mut(dialog_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {dialog_handle} is not a Dialog"),
                "Pass a handle from `Dialog.NewModal`.",
                line,
            ));
        };
        dialog.add(list_box).as_u16()
    };

    worker
        .bridge
        .set_child_parent(list_box_handle, dialog_handle);
    worker
        .bridge
        .registry
        .set_view_id(list_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(list_box_handle), line))?;
    Ok(())
}

/// Attaches a detached list box to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_attach_list_box(
    worker: &mut Worker,
    window_handle: u32,
    list_box_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.bridge.is_on_desktop(window_handle) {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Window handle {window_handle} is already on the desktop"),
            "Call `Window.Add` before `Desktop.Add`.",
            line,
        ));
    }

    worker
        .bridge
        .registry
        .require(window_handle, ViewKind::Window)
        .map_err(|error| registry_error(error, line))?;
    worker
        .bridge
        .registry
        .require(list_box_handle, ViewKind::ListBox)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedListBox { list_box, .. }) =
        worker.bridge.take_detached_list_box(list_box_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("ListBox handle {list_box_handle} is not detached"),
            "Pass a handle from `ListBox.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let view_id = {
        let Some(TuiRoot::Window(window)) = worker.bridge.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        window.add(list_box).as_u16()
    };

    worker
        .bridge
        .set_child_parent(list_box_handle, window_handle);
    worker
        .bridge
        .registry
        .set_view_id(list_box_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(list_box_handle), line))?;
    Ok(())
}

fn list_box_with_items(
    bounds: Rect,
    items: Vec<String>,
    command: u16,
    selection: Option<usize>,
) -> ListBox {
    let mut list_box = ListBox::new(bounds, command);
    list_box.set_items(items);
    if let Some(selection) = selection {
        list_box.set_selection(selection);
    }
    list_box
}

fn missing_list_box_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("ListBox handle {handle} has no host state"),
        "Use a handle returned by `ListBox.New` in the active Turbo Vision session.",
        line,
    )
}

fn missing_list_box_view(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("ListBox handle {handle} is not backed by an upstream ListBox"),
        "Use a handle returned by `ListBox.New` in the active Turbo Vision session.",
        line,
    )
}

fn bridge_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "TUI session is not open",
        "Call `Application.New` before creating Turbo Vision widgets on the Turbo Vision path.",
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
    use crate::vm::execute::io::tui::bridge::registry::ViewKind;
    use crate::vm::execute::io::tui::bridge::views::dialog::bridge_dialog_new_modal;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

    #[test]
    fn list_box_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let handle = bridge_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into(), "beta".into()],
            100,
            loc(),
        )
        .expect("list box");
        assert_eq!(
            worker
                .bridge
                .registry
                .require(handle, ViewKind::ListBox)
                .unwrap()
                .kind,
            ViewKind::ListBox
        );
        assert_eq!(
            bridge_list_box_selection(&mut worker, handle, loc()).unwrap(),
            0
        );
    }

    #[test]
    fn set_items_updates_selection() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let handle = bridge_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into()],
            100,
            loc(),
        )
        .expect("list box");
        bridge_list_box_set_items(&mut worker, handle, vec![], loc()).expect("clear items");
        assert_eq!(
            bridge_list_box_selection(&mut worker, handle, loc()).unwrap(),
            -1
        );
        bridge_list_box_set_items(&mut worker, handle, vec!["one".into()], loc())
            .expect("set one item");
        assert_eq!(
            bridge_list_box_selection(&mut worker, handle, loc()).unwrap(),
            0
        );
    }

    #[test]
    fn dialog_add_list_box_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let dialog =
            bridge_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let list_box = bridge_list_box_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            vec!["alpha".into(), "beta".into()],
            100,
            loc(),
        )
        .expect("list box");
        bridge_dialog_attach_list_box(&mut worker, dialog, list_box, loc()).expect("attach");
        bridge_list_box_set_items(&mut worker, list_box, vec!["updated".into()], loc())
            .expect("set attached items");
        assert_eq!(
            bridge_list_box_selection(&mut worker, list_box, loc()).unwrap(),
            0
        );
        let entry = worker
            .bridge
            .registry
            .require(list_box, ViewKind::ListBox)
            .expect("list box entry");
        assert_ne!(entry.view_id, 0);
    }
}
