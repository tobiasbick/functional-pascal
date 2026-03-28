use super::super::diagnostics::STACK_OVERFLOW_CODE;
use super::super::{STACK_MAX, Vm, VmError, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};

impl Vm {
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

    pub(in super::super) fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.stack.len() >= STACK_MAX {
            return Err(runtime_error(
                STACK_OVERFLOW_CODE,
                "Stack overflow",
                "Reduce recursion depth or intermediate stack usage in this expression.",
                self.current_location,
            ));
        }
        self.stack.push(value);
        Ok(())
    }

    pub(in super::super) fn pop(&mut self, location: SourceLocation) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| {
            internal_error(
                "Stack underflow",
                "This indicates invalid bytecode or a VM bug. Please report it.",
                location,
            )
        })
    }

    pub(in super::super) fn peek(&self, location: SourceLocation) -> Result<&Value, VmError> {
        self.stack.last().ok_or_else(|| {
            internal_error(
                "Stack underflow",
                "This indicates invalid bytecode or a VM bug. Please report it.",
                location,
            )
        })
    }
}
