//! Borrowed file-call arguments and complete bounded byte validation.

use crate::Value;
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::failure::Failure;
use super::state::MAX_BYTES;
use crate::execution::vm::{VmError, worker::Worker};

pub(super) fn count(worker: &Worker, values: &[Value], expected: usize) -> Result<(), VmError> {
    if values.len() == expected {
        return Ok(());
    }
    Err(worker.runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Std.Io.Fs intrinsic expected {expected} arguments, got {}",
            values.len()
        ),
        "Check the intrinsic signature and register argument count.",
    ))
}

pub(super) fn text<'a>(worker: &Worker, value: &'a Value, name: &str) -> Result<&'a str, VmError> {
    match value {
        Value::Str(text) => Ok(text),
        other => Err(mismatch(worker, name, "string", other)),
    }
}

pub(super) fn integer(worker: &Worker, value: &Value, name: &str) -> Result<i64, VmError> {
    match value {
        Value::Integer(number) => Ok(*number),
        other => Err(mismatch(worker, name, "integer", other)),
    }
}

pub(super) fn handle(worker: &Worker, value: &Value, name: &str) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        other => Err(mismatch(worker, name, "opaque handle", other)),
    }
}

pub(super) fn ordinal(value: &Value, mode: bool) -> Result<i64, Failure> {
    let name = if mode { "Mode" } else { "Origin" };
    let maximum = if mode { 3 } else { 2 };
    // Payload-free enums lower to integer ordinals; sema retains nominal admission.
    let Value::Integer(value) = value else {
        return Err(Failure::argument(
            name,
            "Expected a payload-free enum ordinal",
        ));
    };
    if !(0..=maximum).contains(value) {
        return Err(Failure::argument(name, format!("Invalid {name} ordinal")));
    }
    Ok(*value)
}

pub(super) fn bytes(
    worker: &Worker,
    value: &Value,
) -> Result<Result<crate::SharedBytes, Failure>, VmError> {
    let Value::Bytes(bytes) = value else {
        return Err(mismatch(worker, "Bytes", "Std.Bytes.ByteBuffer", value));
    };
    if bytes.len() > MAX_BYTES {
        return Ok(Err(Failure::Limit(MAX_BYTES as i64)));
    }
    Ok(Ok(bytes.clone()))
}

fn mismatch(worker: &Worker, name: &str, expected: &str, actual: &Value) -> VmError {
    worker.runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Io.Fs {name} expected {expected}, got {}",
            actual.type_name()
        ),
        "Pass values matching the documented Std.Io.Fs function signature.",
    )
}
