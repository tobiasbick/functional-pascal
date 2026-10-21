//! Iterative teardown of uniquely owned aggregate payloads.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{ReferenceRoot, Value};
use std::ops::Deref;
use std::sync::Arc;

/// Release owned runtime values without recursively dropping their aggregate or wrapper chains.
///
/// Shared aliases retain their values. This does not collect cycles; it only drains payloads
/// whose final strong owner is released.
pub fn release_values(mut owned: Vec<Value>) {
    values(&mut owned);
}

/// Start teardown without allocating a work buffer for primitive leaves.
pub(super) fn release_value(value: Value) {
    let mut pending = Vec::new();
    enqueue(value, &mut pending);
    values(&mut pending);
}

/// An incoming value whose rejection must release a deep graph iteratively.
pub(super) struct PendingValue(Option<Value>);

impl PendingValue {
    pub(super) fn new(value: Value) -> Self {
        Self(Some(value))
    }

    pub(super) fn into_inner(mut self) -> Value {
        let Some(value) = self.0.take() else {
            unreachable!("a pending value is consumed exactly once");
        };
        value
    }
}

impl Deref for PendingValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        let Some(value) = self.0.as_ref() else {
            unreachable!("a pending value is consumed exactly once");
        };
        value
    }
}

impl Drop for PendingValue {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            release_value(value);
        }
    }
}

pub(super) fn values(pending: &mut Vec<Value>) {
    while let Some(value) = pending.pop() {
        enqueue(value, pending);
    }
}

pub(super) fn pairs(pairs: &mut Vec<(Value, Value)>) {
    let mut pending = Vec::new();
    while let Some((key, value)) = pairs.pop() {
        enqueue(key, &mut pending);
        enqueue(value, &mut pending);
        values(&mut pending);
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
        Value::Cell(cell) => cell.drain_unique(pending),
        Value::Reference(reference) => {
            if let Some(reference) = Arc::into_inner(reference)
                && let ReferenceRoot::Cell(cell) = reference.root
            {
                cell.drain_unique(pending);
            }
        }
        _ => {}
    }
}
