//! Bounded binary file reads and fully validated byte-array writes.

use std::fs::File;
use std::io::Read;

use crate::{SourceLocation, Value};

use super::failure::Failure;
use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::pop_int;

pub(super) const MAX_BYTES: usize = 1_048_576;

#[cfg(test)]
pub(super) fn read_bytes(path: &str) -> Result<Vec<u8>, Failure> {
    read_bytes_controlled(path, &mut || false)
}

pub(super) fn read_bytes_controlled(
    path: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Vec<u8>, Failure> {
    super::io::check(cancelled)?;
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_BYTES as u64 {
        return Err(limit("Std.Io.Fs.ReadBytes"));
    }
    read_limited_controlled(file, cancelled)
}

#[cfg(test)]
pub(super) fn read_limited(reader: impl Read) -> Result<Vec<u8>, Failure> {
    read_limited_controlled(reader, &mut || false)
}

fn read_limited_controlled(
    reader: impl Read,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Vec<u8>, Failure> {
    // The handle can grow or represent a stream whose metadata reports no size.
    let bytes = super::io::read_prefix(reader, (MAX_BYTES + 1) as u64, cancelled)?;
    if bytes.len() > MAX_BYTES {
        return Err(limit("Std.Io.Fs.ReadBytes"));
    }
    Ok(bytes)
}

#[cfg(test)]
pub(super) fn write_bytes(
    path: &str,
    values: &[Value],
    location: SourceLocation,
) -> Result<Result<(), Failure>, StdError> {
    Ok(validate_bytes(values, location)?
        .and_then(|bytes| std::fs::write(path, bytes).map_err(Into::into)))
}

pub(super) fn validate_bytes(
    values: &[Value],
    location: SourceLocation,
) -> Result<Result<Vec<u8>, Failure>, StdError> {
    if values.len() > MAX_BYTES {
        return Ok(Err(limit("Std.Io.Fs.WriteBytes")));
    }
    for (offset, value) in values.iter().enumerate() {
        let value = pop_int(value, location)?;
        if !(0..=255).contains(&value) {
            return Ok(Err(Failure::InvalidByte { offset, value }));
        }
    }
    // No conversion allocation or target access occurs before every byte is valid.
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        bytes.push(pop_int(value, location)? as u8);
    }
    Ok(Ok(bytes))
}

fn limit(operation: &'static str) -> Failure {
    Failure::LimitExceeded(operation, MAX_BYTES as i64)
}
