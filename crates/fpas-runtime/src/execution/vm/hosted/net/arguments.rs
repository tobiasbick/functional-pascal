//! Typed argument validation for hosted network intrinsics.

use super::{VmError, Worker};
use crate::Value;
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
/// Validate argument count before indexing values.
pub(super) fn require_count(
    worker: &Worker,
    arguments: &[Value],
    expected: usize,
) -> Result<(), VmError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(worker.runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Std.Net intrinsic expected {expected} arguments, got {}",
            arguments.len()
        ),
        "Check the compiler intrinsic signature and register argument count.",
    ))
}

/// Read one string argument.
pub(super) fn string<'a>(
    worker: &Worker,
    value: &'a Value,
    name: &str,
) -> Result<&'a str, VmError> {
    match value {
        Value::Str(value) => Ok(value),
        actual => Err(type_error(worker, name, "string", actual)),
    }
}

/// Read one integer argument.
pub(super) fn integer(worker: &Worker, value: &Value, name: &str) -> Result<i64, VmError> {
    match value {
        Value::Integer(value) => Ok(*value),
        actual => Err(type_error(worker, name, "integer", actual)),
    }
}

/// Read an opaque connection handle.
pub(super) fn connection(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(
            worker,
            "Connection",
            "Std.Net.Connection",
            actual,
        )),
    }
}

/// Read an opaque listener handle.
pub(super) fn listener(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(worker, "Listener", "Std.Net.Listener", actual)),
    }
}

/// Read an opaque cancellation token.
pub(super) fn cancellation_token(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(
            worker,
            "Token",
            "Std.Tasks.CancellationToken",
            actual,
        )),
    }
}

/// Validate and convert a byte-array argument.
pub(super) fn bytes(
    worker: &Worker,
    value: &Value,
) -> Result<Result<Vec<u8>, super::failure::Failure>, VmError> {
    let Value::Array(values) = value else {
        return Err(type_error(worker, "Data", "array of integer", value));
    };
    let mut bytes = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let Value::Integer(value) = value else {
            return Err(type_error(worker, "Data element", "integer", value));
        };
        let Ok(byte) = u8::try_from(*value) else {
            return Ok(Err(super::failure::Failure::argument(
                "Data",
                format!("Data[{index}] must be in 0..=255, got {value}"),
            )));
        };
        bytes.push(byte);
    }
    Ok(Ok(bytes))
}

fn type_error(worker: &Worker, name: &str, expected: &str, actual: &Value) -> VmError {
    worker.runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Net {name} expected {expected}, got {}",
            actual.type_name()
        ),
        "Pass values matching the documented Std.Net function signature.",
    )
}
