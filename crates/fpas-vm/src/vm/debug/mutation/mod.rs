//! Atomic live-root mutation after detached expression evaluation.

mod dictionary;
pub(in crate::vm::debug) mod empty_storage;
mod function_value;
mod model;
mod replace;
mod resolve;
mod sequence;
mod transition;
mod validate;
mod variant;

use std::sync::TryLockError;

use fpas_bytecode::{Value, VerifiedExecutable};

use super::inspection::{MutationRoot, MutationTarget};
use super::types::{DebugErrorKind, DebugSessionError};
use crate::vm::worker::Worker;

pub(in crate::vm::debug) use dictionary::{DictionaryTransformation, insert, remove, replace_key};
pub use empty_storage::DebugStorageInitializationResult;
pub(in crate::vm::debug) use function_value::{
    inactive_function_payload, is_function_type, prepare as prepare_function_value,
    source_name as function_value_source_name,
};
pub use model::{
    DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugDictionaryMutationResult, DebugStringMutationResult,
};
pub(in crate::vm::debug) use resolve::{ResolvedAssignment, resolve_assignment, target_with_value};
pub(in crate::vm::debug) use sequence::{insert_array, remove_array, replace_string_character};
pub(in crate::vm::debug) use transition::construct as construct_transition;
pub use variant::{
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
};
pub(in crate::vm::debug) use variant::{
    VariantMetadata, WrapperMetadata, complete_value, constructible_description,
    ordered_field_expressions, require_constructible_fields, require_wrapper, unknown_variant,
};

pub(in crate::vm::debug) fn validate_replacement(
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
pub(in crate::vm::debug) fn validate_value(
    executable: &VerifiedExecutable,
    expected: fpas_bytecode::DebugTypeId,
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    validate::value(executable.executable(), expected, value, max_depth)
}

pub(in crate::vm::debug) fn commit(
    worker: &mut Worker,
    generation: u32,
    target: &MutationTarget,
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    if target.generation != generation {
        return Err(expired());
    }
    if !target.path.is_empty() && !target.initialized {
        return Err(uninitialized_path());
    }
    match &target.root {
        MutationRoot::FrameRegister(register) => {
            if target.path.is_empty() {
                worker
                    .store_register(*register, replacement.clone())
                    .map_err(|_| unavailable())?;
                return Ok(replacement);
            }
            if !worker.register_is_initialized(*register) {
                return Err(uninitialized_path());
            }
            let root = worker
                .registers
                .get(*register)
                .cloned()
                .ok_or_else(unavailable)?;
            let updated = replace::descendant(root, &target.path, replacement)?;
            worker
                .store_register(*register, updated.clone())
                .map_err(|_| unavailable())?;
            Ok(replaced_value(&updated, &target.path).unwrap_or(updated))
        }
        MutationRoot::Global(global) => {
            let mut globals = worker.globals.write().map_err(|_| unavailable())?;
            let slot = globals.get_mut(*global).ok_or_else(unavailable)?;
            if target.path.is_empty() {
                *slot = Some(replacement.clone());
                return Ok(replacement);
            }
            let current = slot.as_mut().ok_or_else(uninitialized_path)?;
            let updated = replace::descendant(current.clone(), &target.path, replacement)?;
            *current = updated.clone();
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

fn uninitialized_path() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: "debug variable target path is unsupported on uninitialized storage".to_string(),
        hint: "Initialize the complete binding before editing fields, indexes, or payload descendants."
            .to_string(),
    }
}
