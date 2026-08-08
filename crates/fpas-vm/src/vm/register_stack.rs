//! Reusable high-water storage for active register windows.

use fpas_bytecode::Value;

use super::worker::Worker;

impl Worker {
    /// Activate a register prefix, retaining already-cleared high-water storage.
    pub(super) fn activate_registers(&mut self, active_count: usize) {
        if active_count < self.active_register_count {
            self.release_registers(active_count);
            return;
        }
        if self.registers.len() < active_count {
            self.registers.resize(active_count, Value::Unit);
        }
        debug_assert!(
            self.registers[self.active_register_count..active_count]
                .iter()
                .all(|value| matches!(value, Value::Unit)),
            "inactive register slots must be cleared before reuse"
        );
        self.active_register_count = active_count;
    }

    /// Release active slots and clear their values without shrinking physical storage.
    pub(super) fn release_registers(&mut self, active_count: usize) {
        debug_assert!(active_count <= self.active_register_count);
        for slot in &mut self.registers[active_count..self.active_register_count] {
            *slot = Value::Unit;
        }
        self.active_register_count = active_count;
    }

    /// Clear the current frame stack and activate a fresh root register prefix.
    pub(super) fn reset_registers(&mut self, active_count: usize) {
        self.release_registers(0);
        self.activate_registers(active_count);
    }
}
