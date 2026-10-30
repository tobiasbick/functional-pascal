//! Native byte-buffer factories and read-only value operations.

mod materialize;
#[cfg(test)]
mod tests;

use super::{
    error::{StdError, std_runtime_error},
    intrinsic_args::{IntrinsicCall, expect_array, pop_int, pop_value},
};
use crate::{
    ByteStorageError, BytesIntrinsic, Intrinsic, MAX_BYTE_BUFFER_LENGTH, SharedBytes,
    SourceLocation, Value,
};

enum Failure {
    InvalidByte(usize, i64),
    Storage(ByteStorageError),
    ArrayAllocation(u64),
}

impl From<ByteStorageError> for Failure {
    fn from(error: ByteStorageError) -> Self {
        Self::Storage(error)
    }
}

pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Bytes(operation) = intrinsic else {
        return Ok(None);
    };
    if matches!(operation, BytesIntrinsic::Length | BytesIntrinsic::Capacity) {
        let bytes = expect_bytes(pop_value(call, location)?, location)?;
        call.push(Value::Integer(if operation == BytesIntrinsic::Length {
            bytes.len()
        } else {
            bytes.capacity()
        } as i64));
        return Ok(Some(()));
    }
    let outcome: Result<Value, Failure> = match operation {
        BytesIntrinsic::JoinChunks => materialize::join(call, location)?,
        BytesIntrinsic::DecodePercent => materialize::percent(call, location)?,
        BytesIntrinsic::Create => {
            let length = nonnegative(pop_int(pop_value(call, location)?, location)?, location)?;
            SharedBytes::zeroed(length)
                .map(Value::Bytes)
                .map_err(Into::into)
        }
        BytesIntrinsic::FromArray => {
            let values = expect_array(pop_value(call, location)?, location)?;
            from_array(values, location)?
        }
        BytesIntrinsic::ToArray => {
            let bytes = expect_bytes(pop_value(call, location)?, location)?;
            let mut values = Vec::new();
            match values.try_reserve_exact(bytes.len()) {
                Ok(()) => {
                    values.extend(bytes.iter().map(|byte| Value::Integer(i64::from(*byte))));
                    Ok(Value::Array(values.into()))
                }
                Err(_) => Err(Failure::ArrayAllocation(
                    bytes.len() as u64 * std::mem::size_of::<Value>() as u64,
                )),
            }
        }
        BytesIntrinsic::Slice => {
            let length = pop_nonnegative(call, location)?;
            let start = pop_nonnegative(call, location)?;
            let bytes = expect_bytes(pop_value(call, location)?, location)?;
            bytes
                .slice(start, length)
                .map(Value::Bytes)
                .map_err(Into::into)
        }
        BytesIntrinsic::Concat => {
            let other = expect_bytes(pop_value(call, location)?, location)?;
            let mut bytes = expect_bytes(pop_value(call, location)?, location)?.clone();
            bytes
                .concat(other)
                .map(|()| Value::Bytes(bytes))
                .map_err(Into::into)
        }
        BytesIntrinsic::SetAt => {
            let value = pop_int(pop_value(call, location)?, location)?;
            let index = pop_nonnegative(call, location)?;
            let bytes = expect_bytes(pop_value(call, location)?, location)?;
            check_range(bytes.len(), index, 1, location)?;
            match u8::try_from(value) {
                Ok(value) => {
                    let mut bytes = bytes.clone();
                    bytes
                        .set_at(index, value)
                        .map(|()| Value::Bytes(bytes))
                        .map_err(Into::into)
                }
                Err(_) => Err(Failure::InvalidByte(index, value)),
            }
        }
        BytesIntrinsic::Fill => {
            let length = pop_nonnegative(call, location)?;
            let start = pop_nonnegative(call, location)?;
            let value = pop_int(pop_value(call, location)?, location)?;
            let bytes = expect_bytes(pop_value(call, location)?, location)?;
            check_range(bytes.len(), start, length, location)?;
            match u8::try_from(value) {
                Ok(value) => {
                    let mut bytes = bytes.clone();
                    bytes
                        .fill(start, length, value)
                        .map(|()| Value::Bytes(bytes))
                        .map_err(Into::into)
                }
                Err(_) => Err(Failure::InvalidByte(start, value)),
            }
        }
        BytesIntrinsic::CopyFrom => {
            let length = pop_nonnegative(call, location)?;
            let target_start = pop_nonnegative(call, location)?;
            let source_start = pop_nonnegative(call, location)?;
            let source = expect_bytes(pop_value(call, location)?, location)?;
            let mut target = expect_bytes(pop_value(call, location)?, location)?.clone();
            target
                .copy_from(source, source_start, target_start, length)
                .map(|()| Value::Bytes(target))
                .map_err(Into::into)
        }
        BytesIntrinsic::Resize | BytesIntrinsic::Reserve => {
            let length = pop_nonnegative(call, location)?;
            let mut bytes = expect_bytes(pop_value(call, location)?, location)?.clone();
            let result = if operation == BytesIntrinsic::Resize {
                bytes.resize(length)
            } else {
                bytes.reserve(length)
            };
            result.map(|()| Value::Bytes(bytes)).map_err(Into::into)
        }
        BytesIntrinsic::TrimCapacity => {
            let mut bytes = expect_bytes(pop_value(call, location)?, location)?.clone();
            bytes
                .trim_capacity()
                .map(|()| Value::Bytes(bytes))
                .map_err(Into::into)
        }
        BytesIntrinsic::Length | BytesIntrinsic::Capacity => unreachable!("queries return above"),
    };
    let value = match outcome {
        Ok(value) => Value::result_ok(value),
        Err(Failure::Storage(ByteStorageError::InvalidRange)) => return Err(range_error(location)),
        Err(failure) => {
            let (variant, fields) = match failure {
                Failure::InvalidByte(offset, value) => (
                    0,
                    vec![Value::Integer(offset as i64), Value::Integer(value)],
                ),
                Failure::Storage(ByteStorageError::LimitExceeded) => {
                    (1, vec![Value::Integer(MAX_BYTE_BUFFER_LENGTH as i64)])
                }
                Failure::Storage(ByteStorageError::AllocationFailed(bytes)) => {
                    (2, vec![Value::Integer(bytes as i64)])
                }
                Failure::ArrayAllocation(bytes) => (2, vec![Value::Integer(bytes as i64)]),
                Failure::Storage(ByteStorageError::InvalidRange) => {
                    unreachable!("range failures return above")
                }
            };
            Value::result_error(super::native_errors::value(
                call.factory(),
                super::native_errors::NativeErrorFamily::Buffer,
                variant,
                fields,
                location,
            )?)
        }
    };
    call.push(value);
    Ok(Some(()))
}

fn from_array(
    values: &[Value],
    location: SourceLocation,
) -> Result<Result<Value, Failure>, StdError> {
    if values.len() > MAX_BYTE_BUFFER_LENGTH {
        return Ok(Err(ByteStorageError::LimitExceeded.into()));
    }
    for (offset, value) in values.iter().enumerate() {
        let value = pop_int(value, location)?;
        if u8::try_from(value).is_err() {
            return Ok(Err(Failure::InvalidByte(offset, value)));
        }
    }
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(values.len()).is_err() {
        return Ok(Err(ByteStorageError::AllocationFailed(values.len()).into()));
    }
    for value in values {
        bytes.push(pop_int(value, location)? as u8);
    }
    Ok(SharedBytes::from_vec(bytes)
        .map(Value::Bytes)
        .map_err(Into::into))
}

fn expect_bytes(value: &Value, location: SourceLocation) -> Result<&SharedBytes, StdError> {
    match value {
        Value::Bytes(bytes) => Ok(bytes),
        _ => Err(std_runtime_error(
            fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "Expected Std.Bytes.ByteBuffer",
            "Pass a byte-buffer value.",
            location,
        )),
    }
}

fn pop_nonnegative(
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<usize, StdError> {
    nonnegative(pop_int(pop_value(call, location)?, location)?, location)
}

fn nonnegative(value: i64, location: SourceLocation) -> Result<usize, StdError> {
    if value < 0 {
        Err(range_error(location))
    } else {
        // Oversized positive requests must still reach the typed limit check on 32-bit hosts.
        Ok(usize::try_from(value).unwrap_or(usize::MAX))
    }
}

fn check_range(
    total: usize,
    start: usize,
    length: usize,
    location: SourceLocation,
) -> Result<(), StdError> {
    if start > total || length > total - start {
        Err(range_error(location))
    } else {
        Ok(())
    }
}

fn range_error(location: SourceLocation) -> StdError {
    std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
        "Byte buffer index, length, capacity or range is invalid",
        "Use nonnegative lengths and ranges within the buffer's Length().",
        location,
    )
}
