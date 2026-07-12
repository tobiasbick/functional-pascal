//! Turbo Vision bridge `TextViewer` construction, attach, and `SetText`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::super::view_lookup::{bridge_replace_child_view, bridge_with_child_view};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridge::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::bridge::session::{DetachedTextViewer, TuiRoot};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::text_viewer::TextViewer;

/// Creates a detached text viewer (`TextViewer.New`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_text_viewer_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.bridge.is_open() {
        return Err(bridge_session_closed_error(line));
    }

    let text_viewer = Box::new(text_viewer_with_text(bounds, &text));
    Ok(worker
        .bridge
        .insert_detached_text_viewer(text_viewer, bounds, text))
}

/// Replaces text viewer content (`TextViewer.SetText`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_text_viewer_set_text(
    worker: &mut Worker,
    handle: u32,
    text: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .bridge
        .registry
        .require(handle, ViewKind::TextViewer)
        .map_err(|error| registry_error(error, line))?;

    worker.bridge.set_text_viewer_text(handle, text.clone());

    if worker.bridge.child_parent(handle).is_some() {
        let bounds = bridge_with_child_view(worker, handle, ViewKind::TextViewer, line, |view| {
            Ok(view.bounds())
        })?;
        bridge_replace_child_view(
            worker,
            handle,
            ViewKind::TextViewer,
            Box::new(text_viewer_with_text(bounds, &text)),
            line,
        )?;
    } else if let Some(bounds) = worker.bridge.detached_text_viewer_bounds(handle) {
        worker
            .bridge
            .replace_detached_text_viewer(handle, Box::new(text_viewer_with_text(bounds, &text)));
    }

    Ok(())
}

fn text_viewer_with_text(bounds: Rect, text: &str) -> TextViewer {
    let mut text_viewer = TextViewer::new(bounds);
    text_viewer.set_text(text);
    text_viewer
}

/// Attaches a detached text viewer to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dialog_attach_text_viewer(
    worker: &mut Worker,
    dialog_handle: u32,
    text_viewer_handle: u32,
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
        .require(text_viewer_handle, ViewKind::TextViewer)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedTextViewer { text_viewer, .. }) =
        worker.bridge.take_detached_text_viewer(text_viewer_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("TextViewer handle {text_viewer_handle} is not detached"),
            "Pass a handle from `TextViewer.New` that has not been added to a parent yet.",
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
        dialog.add(text_viewer).as_u16()
    };

    worker
        .bridge
        .set_child_parent(text_viewer_handle, dialog_handle);
    worker
        .bridge
        .registry
        .set_view_id(text_viewer_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(text_viewer_handle), line))?;
    Ok(())
}

/// Attaches a detached text viewer to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_window_attach_text_viewer(
    worker: &mut Worker,
    window_handle: u32,
    text_viewer_handle: u32,
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
        .require(text_viewer_handle, ViewKind::TextViewer)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedTextViewer { text_viewer, .. }) =
        worker.bridge.take_detached_text_viewer(text_viewer_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("TextViewer handle {text_viewer_handle} is not detached"),
            "Pass a handle from `TextViewer.New` that has not been added to a parent yet.",
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
        window.add(text_viewer).as_u16()
    };

    worker
        .bridge
        .set_child_parent(text_viewer_handle, window_handle);
    worker
        .bridge
        .registry
        .set_view_id(text_viewer_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(text_viewer_handle), line))?;
    Ok(())
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
            "Use a handle returned by `TextViewer.New` in the active session.",
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
    use crate::vm::execute::io::tui::bridge::registry::ViewKind;
    use crate::vm::execute::io::tui::bridge::views::dialog::bridge_dialog_new_modal;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

    #[test]
    fn text_viewer_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let handle =
            bridge_text_viewer_new(&mut worker, Rect::new(3, 2, 20, 4), "hello".into(), loc())
                .expect("text viewer");
        assert_eq!(
            worker
                .bridge
                .registry
                .require(handle, ViewKind::TextViewer)
                .unwrap()
                .kind,
            ViewKind::TextViewer
        );
        assert_eq!(worker.bridge.text_viewer_text(handle), Some("hello"));
    }

    #[test]
    fn set_text_updates_host_state() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let handle =
            bridge_text_viewer_new(&mut worker, Rect::new(3, 2, 20, 4), "old".into(), loc())
                .expect("text viewer");
        bridge_text_viewer_set_text(&mut worker, handle, "new".into(), loc()).expect("set text");
        assert_eq!(worker.bridge.text_viewer_text(handle), Some("new"));
    }

    #[test]
    fn dialog_add_text_viewer_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.bridge.open();
        let dialog =
            bridge_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let viewer =
            bridge_text_viewer_new(&mut worker, Rect::new(3, 2, 20, 4), "body".into(), loc())
                .expect("text viewer");
        bridge_dialog_attach_text_viewer(&mut worker, dialog, viewer, loc()).expect("attach");
        bridge_text_viewer_set_text(&mut worker, viewer, "updated".into(), loc())
            .expect("set text");
        assert_eq!(worker.bridge.text_viewer_text(viewer), Some("updated"));
    }
}
