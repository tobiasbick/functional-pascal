//! Shared stack pop helpers for intrinsic implementations.

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

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
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected string argument, got {}", other.type_name()),
            "Pass a string value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_int(v: Value, location: SourceLocation) -> Result<i64, StdError> {
    match v {
        Value::Integer(n) => Ok(n),
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
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
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected real argument, got {}", other.type_name()),
            "Pass a real value to this Std.* call.",
            location,
        )),
    }
}

/// Returns the single Unicode scalar in `s`, or an error when `s` is empty or longer than one character.
pub(crate) fn single_char_from_string(s: &str, location: SourceLocation) -> Result<char, StdError> {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        (None, _) => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "Expected a single-character string",
            "Pass a string with exactly one character, for example `'A'`.",
            location,
        )),
        _ => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected a single-character string, got `{s}`"),
            "Pass a string with exactly one character, for example `'A'`.",
            location,
        )),
    }
}

/// Pops a string argument and returns its sole character for APIs that accept one code point.
pub(crate) fn pop_single_char(v: Value, location: SourceLocation) -> Result<char, StdError> {
    let s = pop_string(v, location)?;
    single_char_from_string(&s, location)
}

/// Returns the first character of a non-empty fill string for padding helpers.
pub(crate) fn pad_fill_char(s: &str, location: SourceLocation) -> Result<char, StdError> {
    s.chars().next().ok_or_else(|| {
        std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "Pad fill string must not be empty",
            "Pass a non-empty fill string, for example `' '`.",
            location,
        )
    })
}

pub(crate) fn pop_bool(v: Value, location: SourceLocation) -> Result<bool, StdError> {
    match v {
        Value::Boolean(b) => Ok(b),
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected boolean argument, got {}", other.type_name()),
            "Pass a boolean value to this Std.* call.",
            location,
        )),
    }
}

pub(crate) fn pop_dict(
    v: Value,
    location: SourceLocation,
) -> Result<Vec<(Value, Value)>, StdError> {
    match v {
        Value::Dict(pairs) => Ok(pairs),
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("expected dict, got {}", other.type_name()),
            "Pass a `dict of K to V` value.",
            location,
        )),
    }
}

pub(crate) fn pop_array(v: Value, location: SourceLocation) -> Result<Vec<Value>, StdError> {
    match v {
        Value::Array(a) => Ok(a),
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
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
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!(
                "Join expects an array of strings, got {}",
                other.type_name()
            ),
            "Convert each array element to a string before calling Std.Str.Join.",
            location,
        )),
    }
}
