//! Iterative teardown of uniquely owned aggregate payloads.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{ReferenceRoot, Value};
use std::sync::Arc;

pub(super) fn values(pending: &mut Vec<Value>) {
    while let Some(value) = pending.pop() {
        enqueue(value, pending);
    }
}

fn enqueue(value: Value, pending: &mut Vec<Value>) {
    match value {
        Value::Enum(value) => value.drain_unique(pending),
        Value::Record(value) => value.drain_unique(pending),
        Value::Array(value) => value.drain_unique(pending),
        Value::Dict(value) => value.drain_unique(pending),
        Value::Function(value) => value.drain_unique(pending),
        Value::ResultOk(value) | Value::ResultError(value) | Value::OptionSome(value) => {
            pending.push(value.into_inner())
        }
        Value::Cell(cell) => cell_value(cell, pending),
        Value::Reference(reference) => {
            if let Some(reference) = Arc::into_inner(reference)
                && let ReferenceRoot::Cell(cell) = reference.root
            {
                cell_value(cell, pending);
            }
        }
        _ => {}
    }
}

fn cell_value(cell: Arc<std::sync::Mutex<Value>>, pending: &mut Vec<Value>) {
    if let Some(cell) = Arc::into_inner(cell) {
        pending.push(
            cell.into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }
}
