//! Try-2 `Memo` construction, attach, and `SetText`.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::super::view_lookup::try2_with_child_view;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::bridged_memo::BridgedMemo;
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedMemo, Try2Root};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

/// Creates a detached memo (`Memo.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_memo_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let memo = Box::new(BridgedMemo::new(bounds, &text));
    Ok(worker.try2.insert_detached_memo(memo, bounds, text))
}

/// Replaces memo text (`Memo.SetText`).
pub(in crate::vm::execute::io::tui::try2) fn try2_memo_set_text(
    worker: &mut Worker,
    handle: u32,
    text: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::Memo)
        .map_err(|error| registry_error(error, line))?;

    worker.try2.set_memo_text(handle, text.clone());

    if worker.try2.child_parent(handle).is_some() {
        try2_with_child_view(worker, handle, ViewKind::Memo, line, |view| {
            if let Some(memo) = view.as_any_mut().downcast_mut::<BridgedMemo>() {
                memo.set_text_from_fpas(&text);
            }
            Ok(())
        })?;
    } else if let Some(bounds) = worker.try2.detached_memo_bounds(handle) {
        worker
            .try2
            .replace_detached_memo(handle, Box::new(BridgedMemo::new(bounds, &text)));
    }

    Ok(())
}

/// Attaches a detached memo to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_memo(
    worker: &mut Worker,
    dialog_handle: u32,
    memo_handle: u32,
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
        .require(memo_handle, ViewKind::Memo)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedMemo { memo, .. }) = worker.try2.take_detached_memo(memo_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Memo handle {memo_handle} is not detached"),
            "Pass a handle from `Memo.New` that has not been added to a parent yet.",
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
        dialog.add(memo).as_u16()
    };

    worker.try2.set_child_parent(memo_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(memo_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(memo_handle), line))?;
    Ok(())
}

/// Attaches a detached memo to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_memo(
    worker: &mut Worker,
    window_handle: u32,
    memo_handle: u32,
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
        .require(memo_handle, ViewKind::Memo)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedMemo { memo, .. }) = worker.try2.take_detached_memo(memo_handle) else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Memo handle {memo_handle} is not detached"),
            "Pass a handle from `Memo.New` that has not been added to a parent yet.",
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
        window.add(memo).as_u16()
    };

    worker.try2.set_child_parent(memo_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(memo_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(memo_handle), line))?;
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
            "Use a handle returned by `Memo.New` in the active session.",
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
    fn memo_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_memo_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            "hello".into(),
            loc(),
        )
        .expect("memo");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::Memo)
                .unwrap()
                .kind,
            ViewKind::Memo
        );
        assert_eq!(worker.try2.memo_text(handle), Some("hello"));
    }

    #[test]
    fn set_text_updates_host_state() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle =
            try2_memo_new(&mut worker, Rect::new(3, 2, 20, 4), "old".into(), loc()).expect("memo");
        try2_memo_set_text(&mut worker, handle, "new".into(), loc()).expect("set text");
        assert_eq!(worker.try2.memo_text(handle), Some("new"));
    }

    #[test]
    fn dialog_add_memo_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let memo = try2_memo_new(
            &mut worker,
            Rect::new(3, 2, 20, 4),
            "body".into(),
            loc(),
        )
        .expect("memo");
        try2_dialog_attach_memo(&mut worker, dialog, memo, loc()).expect("attach");
        try2_memo_set_text(&mut worker, memo, "updated".into(), loc()).expect("set text");
        assert_eq!(worker.try2.memo_text(memo), Some("updated"));
    }
}
