//! Read outline selection state from FPAS handles.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::outline_nodes::outline_label_at_flat_index;
use super::tv_geometry::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::shared::{TurboVisionObject, TurboVisionState};
use fpas_bytecode::{SourceLocation, Value};

impl Worker {
    /// Read the flat visible selection index of an `Outline` handle.
    pub(super) fn turbo_vision_outline_selection(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let outline_handle = self.pop_turbo_vision_outline_handle(line)?;
        self.pop_tui_application(line)?;

        let selection =
            self.with_tui(|tui| outline_selection(&tui.turbo_vision, outline_handle, line))?;
        self.push(Value::Integer(selection))
    }

    /// Read the label text of the selected outline node.
    pub(super) fn turbo_vision_outline_selected_text(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let outline_handle = self.pop_turbo_vision_outline_handle(line)?;
        self.pop_tui_application(line)?;

        let text =
            self.with_tui(|tui| outline_selected_text(&tui.turbo_vision, outline_handle, line))?;
        self.push(Value::Str(text))
    }
}

fn outline_selection(
    state: &TurboVisionState,
    handle: u32,
    line: SourceLocation,
) -> Result<i64, VmError> {
    match state.objects.get(&handle) {
        Some(TurboVisionObject::Outline(outline)) => Ok(outline
            .selection_cell
            .read()
            .map(|selection| selection as i64)
            .unwrap_or(-1)),
        _ => Err(unknown_handle_error("Outline", handle, line)),
    }
}

fn outline_selected_text(
    state: &TurboVisionState,
    handle: u32,
    line: SourceLocation,
) -> Result<String, VmError> {
    match state.objects.get(&handle) {
        Some(TurboVisionObject::Outline(outline)) => {
            let Some(index) = outline.selection_cell.read() else {
                return Ok(String::new());
            };
            Ok(outline_label_at_flat_index(&outline.roots, index).unwrap_or_default())
        }
        _ => Err(unknown_handle_error("Outline", handle, line)),
    }
}
