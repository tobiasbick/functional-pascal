//! Native UTF-8 conversion with typed allocation errors and sequence-start offsets.
#[cfg(test)]
mod tests;

use crate::stdlib::{
    error::StdError,
    intrinsic_args::{IntrinsicCall, expect_bytes, expect_str, pop_value},
};
use crate::{
    ByteStorageError, MAX_BYTE_BUFFER_LENGTH, SharedBytes, SourceLocation, StrIntrinsic, Value,
};

pub(super) fn run(
    operation: StrIntrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<(), StdError> {
    let result = match operation {
        StrIntrinsic::Utf8Encode => {
            let text = expect_str(pop_value(call, location)?, location)?;
            buffer_result(SharedBytes::from_slice(text.as_bytes()), call, location)?
        }
        StrIntrinsic::Utf8Append => {
            let text = expect_str(pop_value(call, location)?, location)?;
            let mut buffer = expect_bytes(pop_value(call, location)?, location)?.clone();
            let result = buffer.extend_from_slice(text.as_bytes()).map(|()| buffer);
            buffer_result(result, call, location)?
        }
        StrIntrinsic::Utf8Decode => {
            let buffer = expect_bytes(pop_value(call, location)?, location)?;
            match std::str::from_utf8(buffer) {
                Ok(text) => {
                    let mut decoded = String::new();
                    if decoded.try_reserve_exact(text.len()).is_err() {
                        utf8_error(
                            call,
                            "AllocationFailed",
                            vec![Value::Integer(text.len() as i64)],
                            location,
                        )?
                    } else {
                        decoded.push_str(text);
                        Value::result_ok(Value::Str(decoded.into()))
                    }
                }
                Err(error) => {
                    let mut fields = vec![Value::Integer(error.valid_up_to() as i64)];
                    let variant = if error.error_len().is_some() {
                        fields.push(Value::Str(error.to_string().into()));
                        "InvalidByte"
                    } else {
                        "IncompleteSequence"
                    };
                    utf8_error(call, variant, fields, location)?
                }
            }
        }
        _ => unreachable!("only UTF-8 operations dispatch here"),
    };
    call.push(result);
    Ok(())
}

fn buffer_result(
    result: Result<SharedBytes, ByteStorageError>,
    call: &IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    Ok(match result {
        Ok(bytes) => Value::result_ok(Value::Bytes(bytes)),
        Err(error) => {
            let (variant, requested) = match error {
                ByteStorageError::LimitExceeded => ("LimitExceeded", MAX_BYTE_BUFFER_LENGTH),
                ByteStorageError::AllocationFailed(bytes) => ("AllocationFailed", bytes),
                ByteStorageError::InvalidRange => {
                    unreachable!("encoding and append do not select ranges")
                }
            };
            Value::result_error(call.enumeration(
                "Std.Bytes.BufferError",
                variant,
                vec![Value::Integer(requested as i64)],
                location,
            )?)
        }
    })
}

fn utf8_error(
    call: &IntrinsicCall<'_>,
    variant: &str,
    fields: Vec<Value>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    call.enumeration("Std.Text.Utf8.Utf8Error", variant, fields, location)
        .map(Value::result_error)
}
