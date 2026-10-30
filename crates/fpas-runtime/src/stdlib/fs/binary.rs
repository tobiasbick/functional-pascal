//! Bounded binary file reads and trusted native byte writes.

use std::fs::File;
use std::io::Read;

use super::failure::Failure;

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

fn limit(operation: &'static str) -> Failure {
    Failure::LimitExceeded(operation, MAX_BYTES as i64)
}
