use crate::vm::{VmError, Worker, internal_error};
use fpas_bytecode::{SourceLocation, Value};

impl Worker {
    pub(in crate::vm) fn frame_base(&self) -> usize {
        self.call_stack
            .last()
            .map(|frame| frame.base_slot)
            .unwrap_or(0)
    }

    /// Absolute stack index for local at `(enclosing_depth, slot)` (same convention as `GetEnclosing`).
    pub(in crate::vm) fn local_abs_index(
        &self,
        depth: u16,
        slot: u16,
        location: SourceLocation,
    ) -> Result<usize, VmError> {
        let frame_count = self.call_stack.len();
        let base = if depth == 0 {
            self.frame_base()
        } else {
            let depth = depth as usize;
            if depth > frame_count {
                return Err(internal_error(
                    format!(
                        "enclosing local depth {depth} is invalid for call stack depth {frame_count}"
                    ),
                    "This indicates invalid bytecode or a compiler closure-layout bug. Please report it.",
                    location,
                ));
            }
            if depth == frame_count {
                0
            } else {
                self.call_stack[frame_count - 1 - depth].base_slot
            }
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

    pub(in crate::vm) fn drain_stack_tail(
        &mut self,
        count: usize,
        location: SourceLocation,
    ) -> Result<Vec<Value>, VmError> {
        if count > self.stack.len() {
            return Err(internal_error(
                format!(
                    "cannot drain {count} value(s) from stack of len {}",
                    self.stack.len()
                ),
                "This indicates invalid bytecode or a VM stack-layout bug. Please report it.",
                location,
            ));
        }

        let start = self.stack.len() - count;
        Ok(self.stack.drain(start..).collect())
    }
}
