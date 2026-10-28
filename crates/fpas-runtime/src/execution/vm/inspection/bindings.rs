//! Initialization-aware register and reference reads for inspection.

use crate::execution::vm::worker::Worker;
use crate::{
    DebugBinding, DebugType, DebugTypeId, ReferenceRoot, ReferenceStep, Value, VariableReference,
};

/// Result of reading storage; a retained runtime value is not an owned value snapshot.
pub(in crate::execution::vm) enum ReadValue {
    /// Initialized storage, retained without copying aggregate contents.
    Available(Value),
    /// Storage exists but has not received a value.
    Uninitialized,
    /// Inspection could not read storage without waiting or inventing data.
    Unavailable(&'static str),
}

impl ReadValue {
    /// Separate a value from its stable unavailable marker for debugger adapters.
    pub(in crate::execution::vm) fn into_parts(self) -> (Option<Value>, Option<&'static str>) {
        match self {
            Self::Available(value) => (Some(value), None),
            Self::Uninitialized => (None, None),
            Self::Unavailable(reason) => (None, Some(reason)),
        }
    }
}

/// Read one source binding, dereferencing `var` parameters without blocking.
pub(in crate::execution::vm) fn read(
    worker: &Worker,
    base: usize,
    binding: &DebugBinding,
) -> ReadValue {
    let register = base.saturating_add(usize::from(binding.register.get()));
    if register >= worker.registers.len() {
        return ReadValue::Unavailable("storage_unavailable");
    }
    if !worker.register_is_initialized(register) {
        return ReadValue::Uninitialized;
    }
    match &worker.registers[register] {
        Value::Reference(reference) => read_reference(worker, reference),
        value => ReadValue::Available(value.clone()),
    }
}

/// Declared value type, unwrapping a `var` parameter's storage reference.
pub(in crate::execution::vm) fn value_type(worker: &Worker, binding: &DebugBinding) -> DebugTypeId {
    match worker
        .executable
        .executable()
        .debug_types
        .get(binding.ty.get() as usize)
    {
        Some(DebugType::Reference(inner)) => *inner,
        _ => binding.ty,
    }
}

/// Read a fixed reference path without executing FPAS or acquiring a blocking lock.
pub(in crate::execution::vm) fn read_reference(
    worker: &Worker,
    reference: &VariableReference,
) -> ReadValue {
    match &reference.root {
        ReferenceRoot::Cell(cell) => match cell.try_read(&worker.domain) {
            Ok(root) => resolve_path(&root, &reference.path),
            Err(crate::OwnershipError::Busy) => ReadValue::Unavailable("busy"),
            Err(crate::OwnershipError::Poisoned) => ReadValue::Unavailable("poisoned"),
            Err(_) => ReadValue::Unavailable("ownership_unavailable"),
        },
        ReferenceRoot::Global { slot, owner } if owner == worker.domain.id() => {
            match super::globals::read(worker, *slot as usize) {
                ReadValue::Available(root) => resolve_path(&root, &reference.path),
                other => other,
            }
        }
        ReferenceRoot::Global { .. } => ReadValue::Unavailable("ownership_unavailable"),
    }
}

fn resolve_path(root: &Value, path: &[ReferenceStep]) -> ReadValue {
    let mut current = root;
    for step in path {
        let child = match (step, current) {
            (ReferenceStep::Field(index), Value::Record(record)) => {
                record.body().values.get(usize::from(*index))
            }
            (ReferenceStep::Element(index), Value::Array(values)) => usize::try_from(*index)
                .ok()
                .and_then(|index| values.get(index)),
            _ => None,
        };
        let Some(child) = child else {
            return ReadValue::Unavailable("storage_unavailable");
        };
        current = child;
    }
    ReadValue::Available(current.clone())
}
