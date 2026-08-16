//! Exact live-frame source initializer suppression.

use fpas_bytecode::{FunctionId, InstructionAddress};

use crate::vm::worker::Worker;

/// One verified initializer store bound to one currently live frame window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::vm) struct SourceInitializerTarget {
    pub function: FunctionId,
    pub base: usize,
    pub instruction: InstructionAddress,
}

impl Worker {
    pub(in crate::vm) fn suppress_source_initializer(&mut self, target: SourceInitializerTarget) {
        if !self.suppressed_initializers.contains(&target) {
            self.suppressed_initializers.push(target);
        }
    }

    pub(in crate::vm) fn take_suppressed_source_initializer(
        &mut self,
        instruction: InstructionAddress,
    ) -> bool {
        let current_function = self.function;
        let current_base = self.base;
        let call_stack = &self.call_stack;
        self.suppressed_initializers.retain(|target| {
            (target.function == current_function && target.base == current_base)
                || call_stack
                    .iter()
                    .any(|frame| frame.function == target.function && frame.base == target.base)
        });
        let Some(index) = self.suppressed_initializers.iter().position(|target| {
            target.function == current_function
                && target.base == current_base
                && target.instruction == instruction
        }) else {
            return false;
        };
        self.suppressed_initializers.swap_remove(index);
        true
    }
}
