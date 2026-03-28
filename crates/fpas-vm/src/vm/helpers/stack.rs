use super::super::{VmError, Worker, internal_error};
use fpas_bytecode::SourceLocation;

impl Worker {
    pub(in super::super) fn frame_base(&self) -> usize {
        self.call_stack
            .last()
            .map(|frame| frame.base_slot)
            .unwrap_or(0)
    }

    /// Absolute stack index for local at `(enclosing_depth, slot)` (same convention as `GetEnclosing`).
    pub(in super::super) fn local_abs_index(
        &self,
        depth: u16,
        slot: u16,
        location: SourceLocation,
    ) -> Result<usize, VmError> {
        let frame_count = self.call_stack.len();
        let base = if depth == 0 {
            self.frame_base()
        } else if (depth as usize) >= frame_count {
            0
        } else {
            self.call_stack[frame_count - 1 - depth as usize].base_slot
        };

        let index = base + slot as usize;
        if index >= self.stack.len() {
            return Err(internal_error(
                format!(
                    "local index {index} out of stack bounds (len {})",
                    self.stack.len()
                ),
                "This indicates a compiler/runtime stack layout bug. Please report it.",
                location,
            ));
        }

        Ok(index)
    }
}
