//! Turbo Vision bridge `Outline` construction, attach, selection read-back, and `SetNodes`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::super::bridged_outline::BridgedOutline;
use super::super::outline_nodes::{initial_outline_selection, outline_label_at_flat_index};
use super::super::view_lookup::bridge_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::bridge::session::{DetachedOutline, TuiRoot};
use crate::vm::shared::TurboVisionOutlineNode;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

/// Creates a detached outline (`Outline.New`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_outline_new(
    worker: &mut Worker,
    bounds: Rect,
    roots: Vec<TurboVisionOutlineNode>,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(bridge_session_closed_error(line));
    }

    let selection = initial_outline_selection(&roots);
    let selection_cell = TurboVisionListSelectionCell::new(selection);
    let outline = Box::new(BridgedOutline::new(bounds, &roots, selection_cell.clone()));
    Ok(worker
        .bridge
        .insert_detached_outline(outline, roots, selection_cell))
}

/// Returns the flat visible selection index, or `-1` when empty (`Outline.Selection`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_outline_selection(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<i64, VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    if worker.bridge.child_parent(handle).is_some() {
        let _ = bridge_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.sync_selection_from_view();
            }
            Ok(())
        });
    }

    Ok(worker
        .bridge
        .outline_selection_cell(handle)
        .ok_or_else(|| missing_outline_state(handle, line))?
        .read()
        .map(|selection| selection as i64)
        .unwrap_or(-1))
}

/// Returns the label of the selected outline node (`Outline.SelectedText`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_outline_selected_text(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<String, VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    if worker.bridge.child_parent(handle).is_some() {
        let _ = bridge_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.sync_selection_from_view();
            }
            Ok(())
        });
    }

    let Some(state) = worker.bridge.outline_state(handle) else {
        return Err(missing_outline_state(handle, line));
    };
    let Some(index) = state.selection_cell.read() else {
        return Ok(String::new());
    };
    Ok(outline_label_at_flat_index(&state.roots, index).unwrap_or_default())
}

/// Replaces outline roots (`Outline.SetNodes`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_outline_set_nodes(
    worker: &mut Worker,
    handle: u32,
    roots: Vec<TurboVisionOutlineNode>,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let selection = initial_outline_selection(&roots);
    let Some(state) = worker.bridge.outline_state_mut(handle) else {
        return Err(missing_outline_state(handle, line));
    };
    state.roots = roots.clone();
    state.selection_cell.set(selection);

    if worker.bridge.child_parent(handle).is_some() {
        bridge_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.set_roots_from_fpas(roots, selection);
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Attaches a detached outline to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_attach_outline(
    worker: &mut Worker,
    dialog_handle: u32,
    outline_handle: u32,
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
        .require(outline_handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedOutline { outline, .. }) = worker.bridge.take_detached_outline(outline_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Outline handle {outline_handle} is not detached"),
            "Pass a handle from `Outline.New` that has not been added to a parent yet.",
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
        dialog.add(outline).as_u16()
    };

    worker
        .bridge
        .set_child_parent(outline_handle, dialog_handle);
    worker
        .bridge
        .registry
        .set_view_id(outline_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(outline_handle), line))?;
    Ok(())
}

/// Attaches a detached outline to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_attach_outline(
    worker: &mut Worker,
    window_handle: u32,
    outline_handle: u32,
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
        .require(outline_handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedOutline { mut outline, .. }) =
        worker.bridge.take_detached_outline(outline_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Outline handle {outline_handle} is not detached"),
            "Pass a handle from `Outline.New` that has not been added to a parent yet.",
            line,
        ));
    };

    use turbo_vision::core::state::{GF_GROW_HI_X, GF_GROW_HI_Y};
    outline.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);

    let view_id = {
        let Some(TuiRoot::Window(window)) = worker.bridge.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        window.add(outline).as_u16()
    };

    worker
        .bridge
        .set_child_parent(outline_handle, window_handle);
    worker
        .bridge
        .registry
        .set_view_id(outline_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(outline_handle), line))?;
    Ok(())
}

fn missing_outline_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("Outline handle {handle} has no host state"),
        "Use a handle returned by `Outline.New` in the active Turbo Vision session.",
        line,
    )
}

fn bridge_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "TUI session is not open",
        "Call `Application.Open` or `Application.OpenForTest` before constructing widgets.",
        line,
    )
}

fn registry_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("Handle {handle} is not live"),
            "Use a handle returned by `Outline.New` in the active session.",
        ),
        RegistryError::WrongKind {
            handle,
            expected,
            actual,
        } => (
            format!("Handle {handle} expected {:?}, got {:?}", expected, actual),
            "Pass an Outline handle.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}
