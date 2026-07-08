//! Try-2 `InputLine` construction, attach, and read-back.
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{DetachedInputLine, Try2Root};
use crate::vm::turbo_vision_input_text_cell::TurboVisionInputTextCell;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::input_line::InputLine;

/// Creates a detached input line (`InputLine.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_input_line_new(
    worker: &mut Worker,
    bounds: Rect,
    text: String,
    max_length: usize,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    if text.len() > max_length {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "InputLine text length {} exceeds MaxLength {max_length}",
                text.len()
            ),
            "Pass a shorter initial text or a larger MaxLength.",
            line,
        ));
    }

    let text_cell = TurboVisionInputTextCell::new(text);
    Ok(worker
        .try2
        .insert_detached_input_line(bounds, text_cell, max_length))
}

/// Returns the host-side text (`InputLine.Text`).
pub(in crate::vm::execute::io::tui::try2) fn try2_input_line_text(
    worker: &mut Worker,
    handle: u32,
    line: SourceLocation,
) -> Result<String, VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::InputLine)
        .map_err(|error| registry_error(error, line))?;

    worker.try2.commit_input_line_text(handle);
    Ok(worker
        .try2
        .input_line_cell(handle)
        .ok_or_else(|| missing_input_line_cell(handle, line))?
        .read())
}

/// Replaces the input text (`InputLine.SetText`).
pub(in crate::vm::execute::io::tui::try2) fn try2_input_line_set_text(
    worker: &mut Worker,
    handle: u32,
    text: String,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(handle, ViewKind::InputLine)
        .map_err(|error| registry_error(error, line))?;

    let max_length = worker
        .try2
        .input_line_max_length(handle)
        .ok_or_else(|| missing_input_line_cell(handle, line))?;
    if text.len() > max_length {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "InputLine text length {} exceeds MaxLength {max_length}",
                text.len()
            ),
            "Pass a shorter text or recreate the input line with a larger MaxLength.",
            line,
        ));
    }

    let Some(cell) = worker.try2.input_line_cell(handle) else {
        return Err(missing_input_line_cell(handle, line));
    };
    cell.set(text.clone());

    if worker.try2.child_parent(handle).is_some() {
        if let Some(binding) = worker.try2.input_line_binding(handle) {
            *binding.borrow_mut() = text;
        }
    }

    Ok(())
}

/// Attaches a detached input line to a modal dialog (`Dialog.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_dialog_attach_input_line(
    worker: &mut Worker,
    dialog_handle: u32,
    input_line_handle: u32,
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
        .require(input_line_handle, ViewKind::InputLine)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedInputLine {
        local_bounds,
        text_cell,
        max_length,
    }) = worker.try2.take_detached_input_line(input_line_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("InputLine handle {input_line_handle} is not detached"),
            "Pass a handle from `InputLine.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let binding = text_cell.view_binding();
    worker
        .try2
        .set_input_line_binding(input_line_handle, binding.clone());
    let input_line = InputLine::new(local_bounds, max_length, binding);

    let view_id = {
        let Some(Try2Root::ModalDialog(dialog)) = worker.try2.root_mut(dialog_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {dialog_handle} is not a Dialog"),
                "Pass a handle from `Dialog.NewModal`.",
                line,
            ));
        };
        dialog.add(Box::new(input_line)).as_u16()
    };

    worker
        .try2
        .set_child_parent(input_line_handle, dialog_handle);
    worker
        .try2
        .registry
        .set_view_id(input_line_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(input_line_handle), line))?;
    Ok(())
}

/// Attaches a detached input line to a window (`Window.Add`).
pub(in crate::vm::execute::io::tui::try2) fn try2_window_attach_input_line(
    worker: &mut Worker,
    window_handle: u32,
    input_line_handle: u32,
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
        .require(input_line_handle, ViewKind::InputLine)
        .map_err(|error| registry_error(error, line))?;

    let Some(DetachedInputLine {
        local_bounds,
        text_cell,
        max_length,
    }) = worker.try2.take_detached_input_line(input_line_handle)
    else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("InputLine handle {input_line_handle} is not detached"),
            "Pass a handle from `InputLine.New` that has not been added to a parent yet.",
            line,
        ));
    };

    let binding = text_cell.view_binding();
    worker
        .try2
        .set_input_line_binding(input_line_handle, binding.clone());
    let input_line = InputLine::new(local_bounds, max_length, binding);

    let view_id = {
        let Some(Try2Root::Window(window)) = worker.try2.root_mut(window_handle) else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Handle {window_handle} is not a Window"),
                "Pass a handle from `Window.New`.",
                line,
            ));
        };
        window.add(Box::new(input_line)).as_u16()
    };

    worker
        .try2
        .set_child_parent(input_line_handle, window_handle);
    worker
        .try2
        .registry
        .set_view_id(input_line_handle, view_id)
        .map_err(|_| registry_error(RegistryError::UnknownHandle(input_line_handle), line))?;
    Ok(())
}

fn missing_input_line_cell(handle: u32, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("InputLine handle {handle} has no host state"),
        "Use a handle returned by `InputLine.New` in the active try-2 session.",
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
            "Use a handle returned by `InputLine.New` in the active session.",
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
    fn input_line_new_registers_handle() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle = try2_input_line_new(
            &mut worker,
            Rect::new(2, 1, 18, 1),
            "FPAS".into(),
            32,
            loc(),
        )
        .expect("input line");
        assert_eq!(
            worker
                .try2
                .registry
                .require(handle, ViewKind::InputLine)
                .unwrap()
                .kind,
            ViewKind::InputLine
        );
        assert_eq!(
            try2_input_line_text(&mut worker, handle, loc()).unwrap(),
            "FPAS"
        );
    }

    #[test]
    fn set_text_updates_host_cell() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let handle =
            try2_input_line_new(&mut worker, Rect::new(2, 1, 18, 1), "old".into(), 32, loc())
                .expect("input line");
        try2_input_line_set_text(&mut worker, handle, "new".into(), loc()).expect("set text");
        assert_eq!(
            try2_input_line_text(&mut worker, handle, loc()).unwrap(),
            "new"
        );
    }

    #[test]
    fn dialog_add_input_line_registers_upstream_view_id() {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.try2.open();
        let dialog =
            try2_dialog_new_modal(&mut worker, Rect::new(5, 3, 30, 8), "Test".into(), loc())
                .expect("dialog");
        let input_line = try2_input_line_new(
            &mut worker,
            Rect::new(4, 2, 18, 1),
            "FPAS".into(),
            32,
            loc(),
        )
        .expect("input line");
        try2_dialog_attach_input_line(&mut worker, dialog, input_line, loc()).expect("attach");
        let entry = worker
            .try2
            .registry
            .require(input_line, ViewKind::InputLine)
            .expect("input line entry");
        assert_ne!(entry.view_id, 0);
    }
}
