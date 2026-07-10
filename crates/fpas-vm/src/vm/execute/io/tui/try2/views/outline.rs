//! Try-2 `Outline` construction, attach, selection read-back, and `SetNodes`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::bridged_outline::BridgedOutline;
use super::super::outline_nodes::{initial_outline_selection, outline_label_at_flat_index};
use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedOutline, Try2Root};
use crate::vm::shared::TurboVisionOutlineNode;
use crate::vm::turbo_vision_list_selection_cell::TurboVisionListSelectionCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

/// Creates a detached outline (`Outline.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_outline_new(
    worker: &mut Worker,
    bounds: Rect,
    roots: Vec<TurboVisionOutlineNode>,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let selection = initial_outline_selection(&roots);
    let selection_cell = TurboVisionListSelectionCell::new(selection);
    let outline = Box::new(BridgedOutline::new(bounds, &roots, selection_cell.clone()));
    Ok(worker
        .try2
        .insert_detached_outline(outline, roots, selection_cell))
}

/// Returns the flat visible selection index, or `-1` when empty (`Outline.Selection`).
pub(in crate::vm::execute::io::tui::try2) fn try2_outline_selection(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<i64, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    if worker.try2.child_parent(handle).is_some() {
        let _ = try2_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.sync_selection_from_view();
            }
            Ok(())
        });
    }

    Ok(worker
        .try2
        .outline_selection_cell(handle)
        .ok_or_else(|| missing_outline_state(handle, line))?
        .read()
        .map(|selection| selection as i64)
        .unwrap_or(-1))
}

/// Returns the label of the selected outline node (`Outline.SelectedText`).
pub(in crate::vm::execute::io::tui::try2) fn try2_outline_selected_text(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<String, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    if worker.try2.child_parent(handle).is_some() {
        let _ = try2_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.sync_selection_from_view();
            }
            Ok(())
        });
    }

    let Some(state) = worker.try2.outline_state(handle) else {
        return Err(missing_outline_state(handle, line));
    };
    let Some(index) = state.selection_cell.read() else {
        return Ok(String::new());
    };
    Ok(outline_label_at_flat_index(&state.roots, index).unwrap_or_default())
}

/// Replaces outline roots (`Outline.SetNodes`).
pub(in crate::vm::execute::io::tui::try2) fn try2_outline_set_nodes(
    worker: &mut Worker,
    handle: u32,
    roots: Vec<TurboVisionOutlineNode>,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let selection = initial_outline_selection(&roots);
    let Some(state) = worker.try2.outline_state_mut(handle) else {
        return Err(missing_outline_state(handle, line));
    };
    state.roots = roots.clone();
    state.selection_cell.set(selection);

    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::Outline, line, |view| {
            if let Some(outline) = view.as_any_mut().downcast_mut::<BridgedOutline>() {
                outline.set_roots_from_fpas(roots, selection);
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Attaches a detached outline to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_outline(
    worker: &mut Worker,
    dialog_handle: u32,
    outline_handle: u32,
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
        .require(outline_handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedOutline { outline, .. }) = worker.try2.take_detached_outline(outline_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Outline handle {outline_handle} is not detached"),
            "Pass a handle from `Outline.New` that has not been added to a parent yet.",
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
        dialog.add(outline).as_u16()
    };

    worker.try2.set_child_parent(outline_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(outline_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(outline_handle), line))?;
    Ok(())
}

/// Attaches a detached outline to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_outline(
    worker: &mut Worker,
    window_handle: u32,
    outline_handle: u32,
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
        .require(outline_handle, ViewKind::Outline)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedOutline { outline, .. }) = worker.try2.take_detached_outline(outline_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Outline handle {outline_handle} is not detached"),
            "Pass a handle from `Outline.New` that has not been added to a parent yet.",
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
        window.add(outline).as_u16()
    };

    worker.try2.set_child_parent(outline_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(outline_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(outline_handle), line))?;
    Ok(())
}

fn missing_outline_state(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("Outline handle {handle} has no host state"),
        "Use a handle returned by `Outline.New` in the active try-2 session.",
        line,
    )
}

fn try2_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Try-2 TUI session is not open",
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
