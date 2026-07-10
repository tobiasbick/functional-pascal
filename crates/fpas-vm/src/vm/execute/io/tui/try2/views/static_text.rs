//! Try-2 `StaticText` construction, parent attachment, and `SetText`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::view_lookup::{try2_replace_child_view, try2_with_child_view};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedStaticText, Try2Root};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::static_text::StaticText;

/// Creates a detached static text label (`StaticText.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_static_text_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let static_text = Box::new(StaticText::new(bounds, &text));
    Ok(worker
        .try2
        .insert_detached_static_text(static_text, bounds, text))
}

/// Replaces static text content (`StaticText.SetText`).
pub(in crate::vm::execute::io::tui::try2) fn try2_static_text_set_text(
    worker: &mut Worker,
    handle: u32,
    text: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::StaticText)
        .map_err(|error| registry_error(error, line))?;

    worker.try2.set_static_text_text(handle, text.clone());

    if worker.try2.child_parent(handle).is_some() {
        let bounds = try2_with_child_view(worker, handle, ViewKind::StaticText, line, |view| {
            Ok(view.bounds())
        })?;
        try2_replace_child_view(
            worker,
            handle,
            ViewKind::StaticText,
            Box::new(StaticText::new(bounds, &text)),
            line,
        )?;
    } else if let Some(bounds) = worker.try2.detached_static_text_bounds(handle) {
        worker
            .try2
            .replace_detached_static_text(handle, Box::new(StaticText::new(bounds, &text)));
    }

    Ok(())
}

/// Attaches a detached static text to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_static_text(
    worker: &mut Worker,
    dialog_handle: u32,
    static_text_handle: u32,
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
        .require(static_text_handle, ViewKind::StaticText)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedStaticText {
        static_text,
        local_bounds: _,
    }) = worker.try2.take_detached_static_text(static_text_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("StaticText handle {static_text_handle} is not detached"),
            "Pass a handle from `StaticText.New` that has not been added to a parent yet.",
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
        dialog.add(static_text).as_u16()
    };

    worker
        .try2
        .registry
        .set_view_id(static_text_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(static_text_handle), line))?;
    worker
        .try2
        .set_child_parent(static_text_handle, dialog_handle);
    Ok(())
}

/// Attaches a detached static text to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_static_text(
    worker: &mut Worker,
    window_handle: u32,
    static_text_handle: u32,
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
        .require(static_text_handle, ViewKind::StaticText)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedStaticText {
        static_text,
        local_bounds: _,
    }) = worker.try2.take_detached_static_text(static_text_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("StaticText handle {static_text_handle} is not detached"),
            "Pass a handle from `StaticText.New` that has not been added to a parent yet.",
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
        window.add(static_text).as_u16()
    };

    worker
        .try2
        .registry
        .set_view_id(static_text_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(static_text_handle), line))?;
    worker
        .try2
        .set_child_parent(static_text_handle, window_handle);
    Ok(())
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
            "Use a handle returned by `StaticText.New` in the active session.",
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
    use crate::vm::execute::io::tui::try2::views::window::try2_window_new;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;

    #[test]
    fn static_text_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle =
            try2_static_text_new(&mut worker, Rect::new(2, 1, 20, 1), "Label".into(), loc())
                .expect("static text");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::StaticText)
                .unwrap()
                .kind,
            ViewKind::StaticText
        );
    }

    #[test]
    fn set_text_updates_host_state() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_static_text_new(&mut worker, Rect::new(3, 2, 20, 1), "OLD".into(), loc())
            .expect("static text");
        try2_static_text_set_text(&mut worker, handle, "NEW".into(), loc()).expect("set text");
        assert_eq!(worker.try2.static_text_text(handle), Some("NEW"));
    }

    #[test]
    fn window_add_static_text_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let window = try2_window_new(&mut worker, Rect::new(5, 3, 30, 10), "Test".into(), loc())
            .expect("window");
        let label =
            try2_static_text_new(&mut worker, Rect::new(4, 2, 20, 1), "Hello".into(), loc())
                .expect("label");
        try2_window_attach_static_text(&mut worker, window, label, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(label, ViewKind::StaticText)
            .expect("label entry");
        assert_ne!(entry.view_id, 0);
    }

    #[test]
    fn dialog_add_static_text_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let label =
            try2_static_text_new(&mut worker, Rect::new(4, 2, 20, 1), "Hello".into(), loc())
                .expect("label");
        try2_dialog_attach_static_text(&mut worker, dialog, label, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(label, ViewKind::StaticText)
            .expect("label entry");
        assert_ne!(entry.view_id, 0);
    }
}
