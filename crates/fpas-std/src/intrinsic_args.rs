//! Borrowed argument decoding shared by stack and register intrinsic callers.

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{SharedArray, SharedStr, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

/// One intrinsic invocation over arguments stored in Pascal evaluation order.
///
/// Arguments are decoded from right to left to preserve the established intrinsic convention
/// without moving values out of register windows. Implementations clone only values they return or
/// mutate.
pub(crate) struct IntrinsicCall<'a> {
    arguments: &'a [Value],
    consumed: usize,
    result: Option<Value>,
}

impl<'a> IntrinsicCall<'a> {
    /// Start decoding one borrowed argument window.
    pub(crate) fn new(arguments: &'a [Value]) -> Self {
        Self {
            arguments,
            consumed: 0,
            result: None,
        }
    }

    /// Record the single value produced by an intrinsic function.
    pub(crate) fn push(&mut self, value: Value) {
        self.result = Some(value);
    }

    /// Finish the invocation and return consumed argument count plus its optional result.
    pub(crate) fn finish(self) -> (usize, Option<Value>) {
        (self.consumed, self.result)
    }
}

pub(crate) fn pop_value<'a>(
    call: &mut IntrinsicCall<'a>,
    location: SourceLocation,
) -> Result<&'a Value, StdError> {
    let index = call.arguments.len().checked_sub(call.consumed + 1);
    let value = index.and_then(|index| call.arguments.get(index));
    let Some(value) = value else {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Intrinsic argument window underflow",
            "Check intrinsic arity and ensure the register argument count matches the call.",
            location,
        ));
    };
    call.consumed += 1;
    Ok(value)
}

pub(crate) fn pop_string(v: &Value, location: SourceLocation) -> Result<String, StdError> {
    Ok(expect_str(v, location)?.as_ref().to_owned())
}

/// Borrows a [`Value::Str`] without copying its UTF-8 buffer.
pub(crate) fn expect_str(v: &Value, location: SourceLocation) -> Result<&SharedStr, StdError> {
    match v {
        Value::Str(s) => Ok(s),
        other => Err(type_error("string", other, location)),
    }
}

pub(crate) fn pop_int(v: &Value, location: SourceLocation) -> Result<i64, StdError> {
    match v {
        Value::Integer(n) => Ok(*n),
        other => Err(type_error("integer", other, location)),
    }
}

pub(crate) fn pop_real(v: &Value, location: SourceLocation) -> Result<f64, StdError> {
    match v {
        Value::Real(n) => Ok(*n),
        other => Err(type_error("real", other, location)),
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

/// Borrows a string argument and returns its sole character.
pub(crate) fn pop_single_char(v: &Value, location: SourceLocation) -> Result<char, StdError> {
    let s = expect_str(v, location)?;
    single_char_from_string(s, location)
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

pub(crate) fn pop_bool(v: &Value, location: SourceLocation) -> Result<bool, StdError> {
    match v {
        Value::Boolean(b) => Ok(*b),
        other => Err(type_error("boolean", other, location)),
    }
}

pub(crate) fn pop_dict(
    v: &Value,
    location: SourceLocation,
) -> Result<Vec<(Value, Value)>, StdError> {
    match v {
        Value::Dict(pairs) => Ok(pairs.iter().cloned().collect()),
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected dictionary argument, got {}", other.type_name()),
            "Pass a `dict of K to V` value.",
            location,
        )),
    }
}

pub(crate) fn pop_array(v: &Value, location: SourceLocation) -> Result<Vec<Value>, StdError> {
    Ok(expect_array(v, location)?.iter().cloned().collect())
}

/// Borrows a [`Value::Array`] without copying its elements.
pub(crate) fn expect_array(v: &Value, location: SourceLocation) -> Result<&SharedArray, StdError> {
    match v {
        Value::Array(a) => Ok(a),
        other => Err(type_error("array", other, location)),
    }
}

pub(crate) fn value_as_string_for_join(
    v: &Value,
    location: SourceLocation,
) -> Result<&str, StdError> {
    match v {
        Value::Str(s) => Ok(s),
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

fn type_error(expected: &str, actual: &Value, location: SourceLocation) -> StdError {
    std_runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("Expected {expected} argument, got {}", actual.type_name()),
        format!("Pass a {expected} value to this Std.* call."),
        location,
    )
}
