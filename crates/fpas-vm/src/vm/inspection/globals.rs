//! Nonblocking selection of worker-local or shared global storage.

use super::bindings::ReadValue;
use crate::vm::{globals::GlobalSlots, worker::Worker};
use fpas_bytecode::Value;
use std::sync::TryLockError;

/// Borrow global storage without waiting for another thread.
pub(in crate::vm) fn slots(worker: &Worker) -> Result<GlobalSlots<'_>, &'static str> {
    if let Some(slots) = worker.local_globals.as_deref() {
        return Ok(GlobalSlots::Local(slots));
    }
    match worker.globals.try_read() {
        Ok(slots) => Ok(GlobalSlots::Shared(slots)),
        Err(TryLockError::Poisoned(poisoned)) => Ok(GlobalSlots::Shared(poisoned.into_inner())),
        Err(TryLockError::WouldBlock) => Err("busy"),
    }
}

/// Read one global while distinguishing empty storage from missing storage.
pub(in crate::vm) fn read(worker: &Worker, index: usize) -> ReadValue {
    match slots(worker) {
        Ok(slots) => read_slot(&slots, index),
        Err(reason) => ReadValue::Unavailable(reason),
    }
}

/// Read an already borrowed slot without cloning unrelated globals.
pub(in crate::vm) fn read_slot(slots: &[Option<Value>], index: usize) -> ReadValue {
    match slots.get(index) {
        Some(Some(value)) => ReadValue::Available(value.clone()),
        Some(None) => ReadValue::Uninitialized,
        None => ReadValue::Unavailable("storage_unavailable"),
    }
}
