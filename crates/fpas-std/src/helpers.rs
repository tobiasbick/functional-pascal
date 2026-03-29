//! Shared stack pop helpers for intrinsic implementations.

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

pub(crate) fn pop_value(
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    stack.pop().ok_or_else(|| {
        std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Intrinsic argument stack underflow",
            "Check intrinsic arity and ensure all required arguments are pushed before the call.",
            location,
        )
    })
}

pub(crate) fn pop_string(v: Value, location: SourceLocation) -> Result<String, StdError> {
    match v {
        Value::Str(s) => Ok(s),
        Value::Char(c) => Ok(c.to_string()),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected string argument, got {}", other.type_name()),
            "Pass a string-compatible value (string or char) to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_int(v: Value, location: SourceLocation) -> Result<i64, StdError> {
    match v {
        Value::Integer(n) => Ok(n),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected integer argument, got {}", other.type_name()),
            "Pass an integer value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_real(v: Value, location: SourceLocation) -> Result<f64, StdError> {
    match v {
        Value::Real(n) => Ok(n),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected real argument, got {}", other.type_name()),
            "Pass a real value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_char(v: Value, location: SourceLocation) -> Result<char, StdError> {
    match v {
        Value::Char(c) => Ok(c),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected char argument, got {}", other.type_name()),
            "Pass a char value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_bool(v: Value, location: SourceLocation) -> Result<bool, StdError> {
    match v {
        Value::Boolean(b) => Ok(b),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected boolean argument, got {}", other.type_name()),
            "Pass a boolean value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_array(v: Value, location: SourceLocation) -> Result<Vec<Value>, StdError> {
    match v {
        Value::Array(a) => Ok(a),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Expected array argument, got {}", other.type_name()),
            "Pass an array value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn value_as_string_for_join(
    v: &Value,
    location: SourceLocation,
) -> Result<String, StdError> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        Value::Char(c) => Ok(c.to_string()),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Join expects an array of strings, got {}",
                other.type_name()
            ),
            "Convert each array element to a string before calling Std.Str.Join.",
            location,
        )),
    }
}
