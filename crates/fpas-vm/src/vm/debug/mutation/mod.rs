//! Atomic live-root mutation after detached expression evaluation.

mod dictionary;
mod model;
mod replace;
mod resolve;
mod validate;

use std::sync::TryLockError;

use fpas_bytecode::{Value, VerifiedExecutable};

use super::inspection::{MutationRoot, MutationTarget};
use super::types::{DebugErrorKind, DebugSessionError};
use crate::vm::worker::Worker;

pub(in crate::vm::debug) use dictionary::{DictionaryTransformation, insert, remove, replace_key};
pub use model::{DebugAssignmentSelector, DebugAssignmentTarget, DebugDictionaryMutationResult};
pub(in crate::vm::debug) use resolve::{target as resolve_target, target_with_value};

pub(super) fn validate_replacement(
    executable: &VerifiedExecutable,
    target: &MutationTarget,
    replacement: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    validate::value(
        executable.executable(),
        target.expected_type,
        replacement,
        max_depth,
    )
}

/// Validates one detached operation value against portable debugger type metadata.
pub(super) fn validate_value(
    executable: &VerifiedExecutable,
    expected: fpas_bytecode::DebugTypeId,
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    validate::value(executable.executable(), expected, value, max_depth)
}

pub(super) fn commit(
    worker: &mut Worker,
    generation: u32,
    target: &MutationTarget,
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    if target.generation != generation {
        return Err(expired());
    }
    match &target.root {
        MutationRoot::FrameRegister(register) => {
            let root = worker
                .registers
                .get(*register)
                .cloned()
                .ok_or_else(unavailable)?;
            let updated = replace::descendant(root, &target.path, replacement)?;
            let slot = worker
                .registers
                .get_mut(*register)
                .ok_or_else(unavailable)?;
            *slot = updated.clone();
            Ok(replaced_value(&updated, &target.path).unwrap_or(updated))
        }
        MutationRoot::Global(global) => {
            let mut globals = worker.globals.write().map_err(|_| unavailable())?;
            let slot = globals
                .get_mut(*global)
                .and_then(Option::as_mut)
                .ok_or_else(uninitialized)?;
            let updated = replace::descendant(slot.clone(), &target.path, replacement)?;
            *slot = updated.clone();
            Ok(replaced_value(&updated, &target.path).unwrap_or(updated))
        }
        MutationRoot::ClosureCell(cell) => {
            let mut inner = match cell.try_lock() {
                Ok(inner) => inner,
                Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                    return Err(unavailable());
                }
            };
            let updated = replace::descendant(inner.clone(), &target.path, replacement)?;
            *inner = updated.clone();
            Ok(replaced_value(&updated, &target.path).unwrap_or(updated))
        }
    }
}

fn replaced_value(root: &Value, path: &[super::inspection::MutationPath]) -> Option<Value> {
    replace::resolve(root, path).cloned()
}

fn expired() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetExpired,
        message: "debug variable target belongs to an expired stop snapshot".to_string(),
        hint: "Request scopes and variables again for the current stop.".to_string(),
    }
}

fn unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable live storage is unavailable".to_string(),
        hint: "Retry at a stable stop after the live storage becomes available.".to_string(),
    }
}

fn uninitialized() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUninitialized,
        message: "debug variable live storage is uninitialized".to_string(),
        hint: "Stop after the binding has received a value.".to_string(),
    }
}
