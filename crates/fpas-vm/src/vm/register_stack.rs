//! Reusable high-water storage for active register windows.

use std::mem;

use fpas_bytecode::Value;

use super::worker::Worker;
use super::{VmError, diagnostics};

impl Worker {
    /// Activate a register prefix, retaining already-cleared high-water storage.
    pub(super) fn activate_registers(&mut self, active_count: usize) {
        if active_count < self.active_register_count {
            self.release_registers(active_count);
            return;
        }
        self.ensure_register_capacity(active_count);
        debug_assert!(
            self.registers[self.active_register_count..active_count]
                .iter()
                .all(|value| matches!(value, Value::Unit)),
            "inactive register slots must be cleared before reuse"
        );
        debug_assert!(
            self.register_initialized[self.active_register_count..active_count]
                .iter()
                .all(|initialized| !*initialized),
            "inactive register slots must be uninitialized before reuse"
        );
        self.active_register_count = active_count;
    }

    /// Release active slots and clear their values without shrinking physical storage.
    pub(super) fn release_registers(&mut self, active_count: usize) {
        debug_assert!(active_count <= self.active_register_count);
        for slot in &mut self.registers[active_count..self.active_register_count] {
            *slot = Value::Unit;
        }
        for slot in &mut self.register_initialized[active_count..self.active_register_count] {
            *slot = false;
        }
        self.active_register_count = active_count;
    }

    /// Clear the current frame stack and activate a fresh root register prefix.
    pub(super) fn reset_registers(&mut self, active_count: usize) {
        self.release_registers(0);
        self.activate_registers(active_count);
    }

    /// Write an absolute active register and mark it initialized.
    pub(super) fn store_register(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        *self.register_slot_mut(index)? = value;
        self.register_initialized[index] = true;
        Ok(())
    }

    /// Remove an absolute active register value and mark the slot uninitialized.
    pub(super) fn take_register(&mut self, index: usize) -> Result<Value, VmError> {
        let value = mem::replace(self.register_slot_mut(index)?, Value::Unit);
        self.register_initialized[index] = false;
        Ok(value)
    }

    /// Whether an active register currently holds an initialized value.
    pub(super) fn register_is_initialized(&self, index: usize) -> bool {
        index < self.active_register_count
            && self
                .register_initialized
                .get(index)
                .copied()
                .unwrap_or(false)
    }

    /// Build a register window whose prefix is initialized and remaining slots are empty.
    pub(super) fn register_window(
        count: usize,
        initialized_prefix: impl IntoIterator<Item = Value>,
    ) -> (Vec<Value>, Vec<bool>) {
        let mut registers = vec![Value::Unit; count];
        let mut register_initialized = vec![false; count];
        for (index, value) in initialized_prefix.into_iter().enumerate() {
            registers[index] = value;
            register_initialized[index] = true;
        }
        (registers, register_initialized)
    }

    fn ensure_register_capacity(&mut self, active_count: usize) {
        if self.registers.len() < active_count {
            self.registers.resize(active_count, Value::Unit);
        }
        if self.register_initialized.len() < active_count {
            self.register_initialized.resize(active_count, false);
        }
    }

    fn register_slot_mut(&mut self, index: usize) -> Result<&mut Value, VmError> {
        let executable = self.executable.executable();
        let address = self.current_address;
        self.registers
            .get_mut(..self.active_register_count)
            .and_then(|registers| registers.get_mut(index))
            .ok_or_else(|| {
                diagnostics::internal(
                    executable,
                    address,
                    format!("Register {index} is outside the initialized frame"),
                )
            })
    }
}
