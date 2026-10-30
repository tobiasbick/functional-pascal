//! Private bulk materialization for source HTTP and URI implementations.
use super::*;
use crate::stdlib::intrinsic_args::expect_bytes;

pub(super) fn join(
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Result<Value, Failure>, StdError> {
    let chunks = expect_array(pop_value(call, location)?, location)?;
    let mut length = 0;
    for chunk in chunks {
        let bytes = expect_bytes(chunk, location)?;
        if bytes.len() > MAX_BYTE_BUFFER_LENGTH - length {
            return Ok(Err(ByteStorageError::LimitExceeded.into()));
        }
        length += bytes.len();
    }
    let mut output = Vec::new();
    if output.try_reserve_exact(length).is_err() {
        return Ok(Err(ByteStorageError::AllocationFailed(length).into()));
    }
    for chunk in chunks {
        output.extend_from_slice(expect_bytes(chunk, location)?);
    }
    Ok(SharedBytes::from_vec(output)
        .map(Value::Bytes)
        .map_err(Into::into))
}

pub(super) fn percent(
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Result<Value, Failure>, StdError> {
    let size = pop_nonnegative(call, location)?;
    let count = pop_nonnegative(call, location)?;
    let start = pop_nonnegative(call, location)?;
    let source = expect_bytes(pop_value(call, location)?, location)?;
    check_range(source.len(), start, count, location)?;
    if size > MAX_BYTE_BUFFER_LENGTH {
        return Ok(Err(ByteStorageError::LimitExceeded.into()));
    }
    let mut output = Vec::new();
    if output.try_reserve_exact(size).is_err() {
        return Ok(Err(ByteStorageError::AllocationFailed(size).into()));
    }
    let mut index = start;
    while index < start + count {
        let value = if source[index] == b'%' {
            if start + count - index < 3 {
                return Err(range_error(location));
            }
            let Some(high) = hex(source[index + 1]) else {
                return Err(range_error(location));
            };
            let Some(low) = hex(source[index + 2]) else {
                return Err(range_error(location));
            };
            index += 3;
            high * 16 + low
        } else {
            let value = source[index];
            index += 1;
            value
        };
        if output.len() >= size {
            return Err(range_error(location));
        }
        output.push(value);
    }
    if output.len() != size {
        return Err(range_error(location));
    }
    Ok(SharedBytes::from_vec(output)
        .map(Value::Bytes)
        .map_err(Into::into))
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'A'..=b'F' => Some(value - b'A' + 10),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
