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
            "Std.Net.Sockets intrinsic expected {expected} arguments, got {}",
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
            "Std.Net.Sockets.Connection",
            actual,
        )),
    }
}

/// Read an opaque listener handle.
pub(super) fn listener(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(
            worker,
            "Listener",
            "Std.Net.Sockets.Listener",
            actual,
        )),
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

/// Retain a trusted byte value as the native request's immutable snapshot.
pub(super) fn bytes(worker: &Worker, value: &Value) -> Result<crate::SharedBytes, VmError> {
    let Value::Bytes(bytes) = value else {
        return Err(type_error(worker, "Data", "Std.Bytes.ByteBuffer", value));
    };
    Ok(bytes.clone())
}

fn type_error(worker: &Worker, name: &str, expected: &str, actual: &Value) -> VmError {
    worker.runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Net.Sockets {name} expected {expected}, got {}",
            actual.type_name()
        ),
        "Pass values matching the documented Std.Net.Sockets function signature.",
    )
}
