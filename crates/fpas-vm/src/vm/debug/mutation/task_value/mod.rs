//! Prepare one eligible task-handle replacement for debugger assignment.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod source;

use fpas_bytecode::{DebugType, DebugTypeId, Executable, Value};

use super::super::evaluation::DebugEvaluationLimits;
use super::super::inspection::InspectionSnapshot;
use super::super::types::{DebugErrorKind, DebugSessionError};
use super::portable_type::{self, TypeLimitWording};

pub(in crate::vm::debug) use source::extract as source;

/// Whether a portable debugger type is a task handle with a declared result type.
pub(in crate::vm::debug) fn is_task_type(executable: &Executable, ty: DebugTypeId) -> bool {
    matches!(
        executable.debug_types.get(ty.get() as usize),
        Some(DebugType::Task(_))
    )
}

/// Validate one evaluated, structurally compatible task handle.
///
/// The destination must already have been resolved as a task-typed target. The source
/// name has already been evaluated once through the ordinary expression budget. Cloning
/// `Value::Task` copies only the runtime ID and does not consult the scheduler.
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
        Some(DebugType::Task(_)) => {}
        Some(_) => {
            return Err(type_error(
                "source binding is not a declared task type",
                "Assign from a binding whose declared type is a task handle, for example `Current := Pending`.",
            ));
        }
        None => {
            return Err(type_error(
                "source binding does not retain portable task type metadata",
                "Assign from a source-declared task binding, not an evaluation-only result.",
            ));
        }
    }
    portable_type::require_equal_bounded(
        &executable.debug_types,
        source_type,
        expected,
        limits.max_depth,
        limits.max_detached_values,
        TypeLimitWording::TASK_RESULT,
    )?;
    match value {
        Value::Task(_) => Ok(value),
        _ => Err(type_error(
            "source binding does not contain a task handle",
            "Stop after the source binding has received a task value, then assign that name.",
        )),
    }
}

/// Inactive-variant construction whose payload would be a task handle.
pub(in crate::vm::debug) fn inactive_task_payload() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: "debug task assignment cannot construct an inactive variant payload".to_string(),
        hint: "Assign into an existing task-typed path such as a mutable local or `optional.value` when Some is already active."
            .to_string(),
    }
}

fn dynamic_endpoint(which: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug task assignment rejects a Dynamic {which}"),
        hint: "Assign between bindings with declared task result types, not Dynamic storage."
            .to_string(),
    }
}

fn type_error(detail: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug task assignment is rejected: {detail}"),
        hint: hint.to_string(),
    }
}
