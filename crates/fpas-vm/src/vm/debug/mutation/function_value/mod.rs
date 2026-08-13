//! Prepare one eligible first-class function replacement for debugger assignment.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod captures;
mod signature;

use fpas_bytecode::{DebugType, DebugTypeId, Executable, SharedFunction, Value};

use super::super::evaluation::{DebugEvaluationLimits, DebugExpression};
use super::super::inspection::InspectionSnapshot;
use super::super::types::{DebugErrorKind, DebugSessionError};

/// Whether a portable debugger type is a first-class function signature.
pub(in crate::vm::debug) fn is_function_type(executable: &Executable, ty: DebugTypeId) -> bool {
    matches!(
        executable.debug_types.get(ty.get() as usize),
        Some(DebugType::Function { .. })
    )
}

/// Require the one supported source-expression shape for function assignment.
pub(in crate::vm::debug) fn source_name(
    expression: &DebugExpression,
) -> Result<&str, DebugSessionError> {
    match expression {
        DebugExpression::Name(name) => Ok(name),
        _ => Err(unsupported_source()),
    }
}

/// Validate one evaluated, structurally compatible, non-task-bound function value.
///
/// The destination must already have been resolved as a function-typed target. The source
/// name has already been evaluated once through the ordinary expression budget. Cloning
/// `Value::Function` shares the existing immutable function storage.
pub(in crate::vm::debug) fn prepare(
    executable: &Executable,
    inspection: &InspectionSnapshot,
    name: &str,
    value: Value,
    expected: DebugTypeId,
    frame_id: Option<u64>,
    limits: DebugEvaluationLimits,
) -> Result<Value, DebugSessionError> {
    if matches!(
        executable.debug_types.get(expected.get() as usize),
        Some(DebugType::Dynamic)
    ) {
        return Err(dynamic_endpoint("destination"));
    }
    let source_type = inspection.resolve_binding_type(frame_id, name)?;
    match executable.debug_types.get(source_type.get() as usize) {
        Some(DebugType::Dynamic) => return Err(dynamic_endpoint("source")),
        Some(DebugType::Function { .. }) => {}
        Some(_) => {
            return Err(type_error(
                "source binding is not a declared function type",
                "Assign from a binding whose declared type is a function or procedure signature.",
            ));
        }
        None => {
            return Err(type_error(
                "source binding does not retain portable function type metadata",
                "Assign from a source-declared function binding, not an evaluation-only result.",
            ));
        }
    }
    signature::require_compatible(&executable.debug_types, source_type, expected)?;
    let Value::Function(function) = &value else {
        return Err(type_error(
            "source binding does not contain a first-class function value",
            "Stop after the source binding has received a function value, then assign that name.",
        ));
    };
    captures::require_eligible(function, limits.max_depth, limits.max_detached_values)?;
    Ok(value)
}

/// Re-check a function replacement at the mutation root before commit.
pub(super) fn validate_root(
    function: &SharedFunction,
    max_depth: usize,
    max_values: usize,
) -> Result<(), DebugSessionError> {
    captures::require_eligible(function, max_depth, max_values)
}

/// Inactive-variant construction whose payload would be a function value.
pub(in crate::vm::debug) fn inactive_function_payload() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: "debug function assignment cannot construct an inactive variant payload"
            .to_string(),
        hint: "Assign into an existing function-typed path such as a mutable local or `optional.value` when Some is already active."
            .to_string(),
    }
}

fn unsupported_source() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: "debug function assignment requires one visible source binding name".to_string(),
        hint:
            "Copy an existing function value with a binding name, for example `Current := Backup`."
                .to_string(),
    }
}

fn dynamic_endpoint(which: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug function assignment rejects a Dynamic {which}"),
        hint: "Assign between bindings with declared function signatures, not Dynamic storage."
            .to_string(),
    }
}

fn type_error(detail: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug function assignment is rejected: {detail}"),
        hint: hint.to_string(),
    }
}
