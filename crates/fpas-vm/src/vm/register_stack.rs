//! Register storage whose length is the active register window.
//!
//! `registers` and `register_initialized` always hold exactly the live prefix of all frames.
//! Releasing a frame truncates both vectors, which keeps their capacity for reuse, so a single
//! length check bounds every register access.

use std::mem;

use fpas_bytecode::Value;

use super::worker::Worker;
use super::{VmError, diagnostics};

impl Worker {
    /// Number of live register slots across all active frames.
    #[inline(always)]
    pub(in crate::vm) fn active_register_count(&self) -> usize {
        self.registers.len()
    }

    /// Activate a register prefix; new slots start as uninitialized `Unit` values.
    pub(super) fn activate_registers(&mut self, active_count: usize) {
        if active_count < self.active_register_count() {
            self.release_registers(active_count);
            return;
        }
        // `resize_with` writes `Unit` directly; `resize` would call the full `Value::clone` per slot,
        // which dominated the cost of entering small frames.
        self.registers.resize_with(active_count, || Value::Unit);
        self.register_initialized.resize(active_count, false);
    }

    /// Release active slots and drop their values without shrinking the allocation.
    pub(super) fn release_registers(&mut self, active_count: usize) {
        debug_assert!(active_count <= self.active_register_count());
        self.registers.truncate(active_count);
        self.register_initialized.truncate(active_count);
    }

    /// Clear the current frame stack and activate a fresh root register prefix.
    pub(super) fn reset_registers(&mut self, active_count: usize) {
        let kept = active_count.min(self.active_register_count());
        self.release_registers(kept);
        // Clear reused slots in place; callbacks reset the same small window repeatedly.
        self.registers.fill(Value::Unit);
        self.register_initialized.fill(false);
        self.activate_registers(active_count);
    }

    /// Write an absolute active register and mark it initialized.
    #[inline(always)]
    pub(super) fn store_register(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        if index >= self.registers.len() || index >= self.register_initialized.len() {
            return Err(self.register_outside_frame(index));
        }
        self.registers[index] = value;
        self.register_initialized[index] = true;
        Ok(())
    }

    /// Write a real to an absolute active register, updating an existing real payload in place.
    ///
    /// Replacing the whole `Value` builds it on the stack and reloads it as one wide value, which
    /// puts every preceding check on the arithmetic dependency chain; the in-place path avoids it.
    #[inline(always)]
    pub(super) fn store_real_register(&mut self, index: usize, value: f64) -> Result<(), VmError> {
        if index >= self.registers.len() || index >= self.register_initialized.len() {
            return Err(self.register_outside_frame(index));
        }
        match &mut self.registers[index] {
            Value::Real(slot) => *slot = value,
            slot => *slot = Value::Real(value),
        }
        self.register_initialized[index] = true;
        Ok(())
    }

    /// Write an integer to an absolute active register, updating an existing integer in place.
    ///
    /// Like [`Self::store_real_register`], this skips building and dropping a whole `Value`
    /// when the slot already holds an integer, as loop counters and accumulators do.
    #[inline(always)]
    pub(super) fn store_integer_register(
        &mut self,
        index: usize,
        value: i64,
    ) -> Result<(), VmError> {
        if index >= self.registers.len() || index >= self.register_initialized.len() {
            return Err(self.register_outside_frame(index));
        }
        match &mut self.registers[index] {
            Value::Integer(slot) => *slot = value,
            slot => *slot = Value::Integer(value),
        }
        self.register_initialized[index] = true;
        Ok(())
    }

    /// Write a Boolean to an absolute active register, updating an existing Boolean in place.
    #[inline(always)]
    pub(super) fn store_boolean_register(
        &mut self,
        index: usize,
        value: bool,
    ) -> Result<(), VmError> {
        if index >= self.registers.len() || index >= self.register_initialized.len() {
            return Err(self.register_outside_frame(index));
        }
        match &mut self.registers[index] {
            Value::Boolean(slot) => *slot = value,
            slot => *slot = Value::Boolean(value),
        }
        self.register_initialized[index] = true;
        Ok(())
    }

    /// Remove an absolute active register value and mark the slot uninitialized.
    #[inline]
    pub(super) fn take_register(&mut self, index: usize) -> Result<Value, VmError> {
        if index >= self.registers.len() || index >= self.register_initialized.len() {
            return Err(self.register_outside_frame(index));
        }
        self.register_initialized[index] = false;
        Ok(mem::replace(&mut self.registers[index], Value::Unit))
    }

    /// Whether an active register currently holds an initialized value.
    pub(super) fn register_is_initialized(&self, index: usize) -> bool {
        self.register_initialized
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

    #[cold]
    #[inline(never)]
    pub(super) fn register_outside_frame(&self, index: usize) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!("Register {index} is outside the initialized frame"),
        )
    }
}
