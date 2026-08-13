//! Eligibility, convention, category, and portable result-type proof.

use fpas_bytecode::{DebugType, DebugTypeId, ReturnConvention, Value, VerifiedExecutable};

use super::super::evaluation::DebugExpression;
use super::super::inspection::DebugFrame;
use super::super::mutation;
use super::super::types::{DebugErrorKind, DebugSessionError, DebugSessionState, DebugStopReason};
use super::{unsupported, value_required, value_unexpected};
use crate::vm::debug::types::DebugTaskState;
use crate::vm::worker::Worker;

/// Reject stops, frames, and tasks outside the accepted forced-return slice.
pub(in crate::vm::debug) fn require_eligible(
    state: DebugSessionState,
    stop_reason: DebugStopReason,
    stop_task_id: u64,
    frame: &DebugFrame,
    frame_task_id: u64,
    task_state: Option<DebugTaskState>,
    worker: &Worker,
) -> Result<(), DebugSessionError> {
    if state == DebugSessionState::Failed || stop_reason == DebugStopReason::RuntimeError {
        return Err(unsupported(
            "forced return is not available after a runtime-error stop",
            "Clear the failure by restarting the debug session; this command is not exception recovery.",
        ));
    }
    if frame_task_id != stop_task_id {
        return Err(unsupported(
            format!(
                "forced return is not available for task {frame_task_id}; the current stop belongs to task {stop_task_id}"
            ),
            "Select the depth-zero frame of the task that caused the current stop.",
        ));
    }
    if frame.depth != 0 {
        return Err(unsupported(
            format!(
                "forced return is not available for frame depth {}; only the active callee is supported",
                frame.depth
            ),
            "Select the current innermost stack frame (depth zero) of the stopped task.",
        ));
    }
    match task_state {
        Some(DebugTaskState::Waiting | DebugTaskState::Sleeping) => {
            return Err(unsupported(
                "forced return is not available for a waiting or sleeping task",
                "Wait until the selected task is the runnable task that caused the current stop.",
            ));
        }
        Some(DebugTaskState::Failed | DebugTaskState::Cancelled | DebugTaskState::Completed) => {
            return Err(unsupported(
                "forced return is not available for a failed, cancelled, or completed task",
                "Select the active callee of the task that caused the current non-failure stop.",
            ));
        }
        Some(DebugTaskState::Runnable | DebugTaskState::Running) | None => {}
    }
    if worker.call_stack.is_empty() {
        return Err(unsupported(
            "forced return is not available for a program or task entry frame",
            "Step into an ordinary callee that has a saved caller before using forced return.",
        ));
    }
    Ok(())
}

/// Require a portable result type before any detached evaluation.
pub(in crate::vm::debug) fn require_result_type(
    result_type: Option<DebugTypeId>,
) -> Result<DebugTypeId, DebugSessionError> {
    result_type.ok_or_else(|| {
        unsupported(
            "forced return is not available because this function has no portable result type metadata",
            "Rebuild the program with current debugger metadata; result types are not inferred from names or opcodes.",
        )
    })
}

/// Require the expression presence to match the callee's return convention.
pub(in crate::vm::debug) fn require_convention(
    convention: ReturnConvention,
    expression: Option<&DebugExpression>,
) -> Result<(), DebugSessionError> {
    match convention {
        ReturnConvention::Unit if expression.is_some() => Err(value_unexpected()),
        ReturnConvention::Value if expression.is_none() => Err(value_required()),
        ReturnConvention::Unit | ReturnConvention::Value => Ok(()),
    }
}

/// Reject declared result categories that this slice does not return.
pub(in crate::vm::debug) fn reject_declared_category(
    executable: &VerifiedExecutable,
    expected: DebugTypeId,
) -> Result<(), DebugSessionError> {
    match declared_type(executable, expected)? {
        DebugType::Dynamic => Err(unsupported(
            "forced return does not support Dynamic results",
            "Return a statically declared scalar or supported aggregate type instead.",
        )),
        DebugType::Function { .. } => Err(unsupported(
            "forced return does not support first-class function results",
            "Use a non-function result type in this first debugger slice.",
        )),
        DebugType::Task(_) => Err(unsupported(
            "forced return does not support task-handle results",
            "Use a non-task result type in this first debugger slice.",
        )),
        DebugType::Cell(_) => Err(unsupported(
            "forced return does not support capture-cell results",
            "Use a non-cell result type in this first debugger slice.",
        )),
        DebugType::Unit
        | DebugType::Boolean
        | DebugType::Integer
        | DebugType::Real
        | DebugType::String
        | DebugType::Array(_)
        | DebugType::Dictionary { .. }
        | DebugType::Result { .. }
        | DebugType::Option(_)
        | DebugType::Record(_)
        | DebugType::Enum(_) => Ok(()),
    }
}

/// Validate one already evaluated or unit value against the declared portable type.
pub(in crate::vm::debug) fn prepare_return_value(
    executable: &VerifiedExecutable,
    result_type: DebugTypeId,
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    match value {
        Value::Function(_) | Value::Task(_) | Value::Cell(_) | Value::OpaqueHandle(_) => {
            return Err(unsupported(
                format!(
                    "forced return does not support {} results",
                    value.type_name()
                ),
                "Supply a detached scalar or supported aggregate without functions, tasks, cells, or opaque handles.",
            ));
        }
        _ => {}
    }
    mutation::validate_value(executable, result_type, value, max_depth).map_err(map_type_error)?;
    Ok(())
}

fn declared_type(
    executable: &VerifiedExecutable,
    expected: DebugTypeId,
) -> Result<&DebugType, DebugSessionError> {
    executable
        .executable()
        .debug_types
        .get(expected.get() as usize)
        .ok_or_else(|| {
            unsupported(
                "forced return is not available because the declared result type is missing",
                "Rebuild the program with current debugger metadata.",
            )
        })
}

fn map_type_error(error: DebugSessionError) -> DebugSessionError {
    if error.kind == DebugErrorKind::VariableValueType {
        DebugSessionError {
            kind: DebugErrorKind::FrameReturnType,
            message: error.message.replacen(
                "debug replacement value",
                "forced return value",
                1,
            ),
            hint: "Use an expression whose complete value matches the function's declared result type."
                .to_string(),
        }
    } else {
        error
    }
}
