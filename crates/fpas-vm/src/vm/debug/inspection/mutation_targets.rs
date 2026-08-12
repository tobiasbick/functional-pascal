//! Handle-based and named lookup of writable stopped-state values.

use std::collections::HashSet;
use std::sync::{Arc, TryLockError};

use fpas_bytecode::Value;

use super::render::RetainedValue;
use super::snapshot::InspectionSnapshot;
use super::targets::{MutationAccess, MutationTarget};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl InspectionSnapshot {
    /// Resolves a writable child previously exposed through a variable handle.
    pub(in crate::vm::debug) fn resolve_mutation_target(
        &self,
        reference: u64,
        name: &str,
    ) -> Result<MutationTarget, DebugSessionError> {
        if (reference >> 32) as u32 != self.generation {
            return Err(DebugSessionError {
                kind: DebugErrorKind::VariableTargetExpired,
                message: format!(
                    "debug variable target `{name}` belongs to an expired stop snapshot"
                ),
                hint: "Request scopes and variables again for the current stop.".to_string(),
            });
        }
        let retained = self
            .handles
            .iter()
            .find(|entry| entry.id == reference)
            .and_then(|entry| {
                entry
                    .values
                    .iter()
                    .find(|value| value.name.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| {
                unknown_target(
                    name,
                    "Request the container variables again and use a returned child name.",
                )
            })?;
        writable_target(retained, name)
    }

    /// Resolves a visible writable root by lexical name or by global-only lookup.
    pub(in crate::vm::debug) fn resolve_named_mutation_target(
        &self,
        frame_id: Option<u64>,
        name: &str,
    ) -> Result<(MutationTarget, Value), DebugSessionError> {
        let frame_values = match frame_id {
            Some(frame_id) => Some(
                self.frames
                    .iter()
                    .find(|frame| frame.frame.id == frame_id)
                    .map(|frame| frame.evaluation_values.as_slice())
                    .ok_or_else(|| DebugSessionError {
                        kind: DebugErrorKind::UnknownFrame,
                        message: format!("debug frame {frame_id} is unknown or expired"),
                        hint: "Request stack frames again for the current stop.".to_string(),
                    })?,
            ),
            None => None,
        };
        let retained = frame_values
            .into_iter()
            .flatten()
            .chain(&self.globals)
            .find(|value| value.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                unknown_target(
                    name,
                    "Use a visible mutable binding; omit frame_id only for globals.",
                )
            })?;
        let target = writable_target(retained, name)?;
        let value = retained.value.clone().ok_or_else(|| uninitialized(name))?;
        Ok((target, materialize(value)?))
    }
}

fn writable_target(
    retained: &RetainedValue,
    name: &str,
) -> Result<MutationTarget, DebugSessionError> {
    if retained.value.is_none() {
        return Err(uninitialized(name));
    }
    match &retained.mutation {
        MutationAccess::Writable(target) => Ok(target.clone()),
        MutationAccess::NotMutable => Err(DebugSessionError {
            kind: DebugErrorKind::VariableNotMutable,
            message: format!("debug variable target `{name}` is not mutable"),
            hint: "Select a source-declared mutable binding or descendant.".to_string(),
        }),
        MutationAccess::Unsupported => Err(DebugSessionError {
            kind: DebugErrorKind::VariablePathUnsupported,
            message: format!("debug variable target `{name}` is not assignable"),
            hint: "Use a mutable binding, stored record field, array element, existing dictionary value, enum payload field, or wrapper `.value`."
                .to_string(),
        }),
        MutationAccess::Unavailable => Err(unavailable()),
    }
}

fn materialize(mut value: Value) -> Result<Value, DebugSessionError> {
    let mut visited = HashSet::new();
    loop {
        let Value::Cell(cell) = value else {
            return Ok(value);
        };
        if !visited.insert(Arc::as_ptr(&cell) as usize) {
            return Err(unavailable());
        }
        value = match cell.try_lock() {
            Ok(inner) => inner.clone(),
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                return Err(unavailable());
            }
        };
    }
}

fn unknown_target(name: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetUnknown,
        message: format!("debug variable target `{name}` does not exist"),
        hint: hint.to_string(),
    }
}

fn uninitialized(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUninitialized,
        message: format!("debug variable target `{name}` is uninitialized"),
        hint: "Stop after the binding has received a value.".to_string(),
    }
}

fn unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable live storage is unavailable".to_string(),
        hint: "Retry at a stable stop after the live storage becomes available.".to_string(),
    }
}
