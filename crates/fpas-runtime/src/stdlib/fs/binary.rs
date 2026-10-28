//! Bounded binary file reads and fully validated byte-array writes.

use std::fs::{self, File};
use std::io::Read;

use crate::{SourceLocation, Value};

use super::failure::Failure;
use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::pop_int;

pub(super) const MAX_BYTES: usize = 1_048_576;

pub(super) fn read_bytes(path: &str) -> Result<Vec<u8>, Failure> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_BYTES as u64 {
        return Err(limit("Std.Io.Fs.ReadBytes"));
    }
    read_limited(file)
}

pub(super) fn read_limited(reader: impl Read) -> Result<Vec<u8>, Failure> {
    // The handle can grow or represent a stream whose metadata reports no size.
    let mut limited = reader.take((MAX_BYTES + 1) as u64);
    let mut bytes = Vec::new();
    limited.read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(limit("Std.Io.Fs.ReadBytes"));
    }
    Ok(bytes)
}

pub(super) fn write_bytes(
    path: &str,
    values: &[Value],
    location: SourceLocation,
) -> Result<Result<(), Failure>, StdError> {
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
    Ok(fs::write(path, bytes).map_err(Into::into))
}

fn limit(operation: &'static str) -> Failure {
    Failure::LimitExceeded(operation, MAX_BYTES as i64)
}
