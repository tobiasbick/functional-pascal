//! Eligibility, detached existing-path resolution, and copy-on-write rebuild.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{DebugTypeId, Executable, Value, VerifiedExecutable};

use super::super::super::inspection::{MutationRoot, MutationTarget};
use super::super::super::types::DebugSessionError;
use super::super::model::{DebugAssignmentSelector, DebugAssignmentTarget};
use super::super::{replace, resolve, validate_value};
use super::diagnostics;
use crate::vm::worker::Worker;

/// Rejects root-only targets, already-initialized storage, and ineligible roots.
pub(in crate::vm::debug) fn require_empty_root(
    assignment: &DebugAssignmentTarget,
    target: &MutationTarget,
) -> Result<(), DebugSessionError> {
    if assignment.selectors.is_empty() {
        return Err(diagnostics::root_only(&assignment.root));
    }
    if target.initialized {
        return Err(diagnostics::already_initialized(&assignment.root));
    }
    match target.root {
        MutationRoot::FrameRegister(_) | MutationRoot::Global(_) => Ok(()),
        MutationRoot::ClosureCell(_) => Err(diagnostics::unsupported_capture(&assignment.root)),
    }
}

/// Validates a complete seed against the declared root type and identity-bearing exclusions.
pub(in crate::vm::debug) fn validate_seed(
    executable: &VerifiedExecutable,
    expected: DebugTypeId,
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    validate_value(executable, expected, value, max_depth)?;
    reject_identity_bearing(value, max_depth)
}

/// Resolves textual selectors against a detached seed without variant transitions or growth.
pub(in crate::vm::debug) fn resolve_existing_path(
    executable: &Executable,
    assignment: &DebugAssignmentTarget,
    target: MutationTarget,
    seed: Value,
    indexes: &[Value],
) -> Result<(MutationTarget, Value), DebugSessionError> {
    resolve::target_with_value(executable, assignment, target, seed, indexes)
}

/// Rebuilds a detached complete root by replacing one existing descendant.
pub(in crate::vm::debug) fn rebuild_root(
    seed: Value,
    path: &[crate::vm::debug::inspection::MutationPath],
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    replace::descendant(seed, path, replacement)
}

/// Whether live storage for an eligible root is still empty.
pub(in crate::vm::debug) fn live_root_is_empty(
    worker: &Worker,
    target: &MutationTarget,
) -> Result<bool, DebugSessionError> {
    match &target.root {
        MutationRoot::FrameRegister(register) => Ok(!worker.register_is_initialized(*register)),
        MutationRoot::Global(global) => {
            let globals = worker
                .globals
                .read()
                .map_err(|_| diagnostics::unavailable())?;
            Ok(globals.get(*global).is_some_and(Option::is_none))
        }
        MutationRoot::ClosureCell(_) => Ok(false),
    }
}

/// Renders a descendant target using requested field names and evaluated indexes.
pub(in crate::vm::debug) fn format_target(
    assignment: &DebugAssignmentTarget,
    indexes: &[Value],
) -> String {
    let mut formatted = assignment.root.clone();
    let mut index = 0;
    for selector in &assignment.selectors {
        match selector {
            DebugAssignmentSelector::Field(name) => {
                formatted.push('.');
                formatted.push_str(name);
            }
            DebugAssignmentSelector::Index(_) => {
                formatted.push('[');
                if let Some(value) = indexes.get(index) {
                    formatted.push_str(&format_index(value));
                }
                formatted.push(']');
                index += 1;
            }
        }
    }
    formatted
}

fn format_index(value: &Value) -> String {
    match value {
        Value::Str(value) => format!("'{}'", value.replace('\'', "''")),
        value => value.to_string(),
    }
}

/// Rejects functions, cells, tasks, and opaque handles anywhere in a seeded value.
pub(in crate::vm::debug) fn reject_identity_bearing(
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    walk_identity(value, max_depth, 0)
}

fn walk_identity(value: &Value, max_depth: usize, depth: usize) -> Result<(), DebugSessionError> {
    if depth > max_depth {
        return Err(diagnostics::identity_bearing(
            "seeded value exceeds the validation depth",
        ));
    }
    match value {
        Value::Function(_) => Err(diagnostics::identity_bearing(
            "function values are not portable empty-storage seeds",
        )),
        Value::Cell(_) => Err(diagnostics::identity_bearing(
            "capture cells are not portable empty-storage seeds",
        )),
        Value::Task(_) => Err(diagnostics::identity_bearing(
            "task handles are not portable empty-storage seeds",
        )),
        Value::OpaqueHandle(_) => Err(diagnostics::identity_bearing(
            "opaque hosted values are not portable empty-storage seeds",
        )),
        Value::Array(values) => values
            .iter()
            .try_for_each(|value| walk_identity(value, max_depth, depth + 1)),
        Value::Dict(entries) => {
            for (key, value) in entries {
                walk_identity(key, max_depth, depth + 1)?;
                walk_identity(value, max_depth, depth + 1)?;
            }
            Ok(())
        }
        Value::Record(record) => record
            .body()
            .values
            .iter()
            .try_for_each(|value| walk_identity(value, max_depth, depth + 1)),
        Value::Enum(enumeration) => enumeration
            .body()
            .values
            .iter()
            .try_for_each(|value| walk_identity(value, max_depth, depth + 1)),
        Value::ResultOk(value) | Value::ResultError(value) | Value::OptionSome(value) => {
            walk_identity(value, max_depth, depth + 1)
        }
        Value::Integer(_)
        | Value::Real(_)
        | Value::Boolean(_)
        | Value::Str(_)
        | Value::Unit
        | Value::OptionNone => Ok(()),
    }
}
