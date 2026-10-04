//! Read stopped storage through its current caller-reference authority.

use crate::vm::worker::Worker;
use fpas_bytecode::{DebugType, DebugTypeId, Executable, Value};

/// Retain a logical value only when this stopped view has read authority.
pub(super) fn snapshot(worker: &Worker, value: &Value) -> Option<Value> {
    match value {
        Value::Cell(cell) => worker.hosted.references.read(cell).ok(),
        Value::Reference(reference) => reference.read().ok(),
        value => Some(value.clone()),
    }
}

/// Remove the runtime cell wrapper from a global's source-visible type.
pub(in crate::vm::debug) fn logical_global_type(
    image: &Executable,
    ty: DebugTypeId,
) -> DebugTypeId {
    match image.debug_types.get(ty.get() as usize) {
        Some(DebugType::Cell(inner)) => *inner,
        _ => ty,
    }
}
