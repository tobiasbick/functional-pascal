//! Cooperative boundaries around native reads and complete writes.

use super::failure::Failure;
use std::io::{self, Read, Write};

pub(super) fn check(cancelled: &mut dyn FnMut() -> bool) -> Result<(), Failure> {
    if cancelled() {
        Err(Failure::Cancelled)
    } else {
        Ok(())
    }
}

pub(super) fn read_prefix(
    reader: impl Read,
    limit: u64,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Vec<u8>, Failure> {
    let mut reader = reader.take(limit);
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        check(cancelled)?;
        match reader.read(&mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

pub(super) fn write_all(
    writer: &mut impl Write,
    bytes: &[u8],
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<(), Failure> {
    for mut chunk in bytes.chunks(8192) {
        while !chunk.is_empty() {
            check(cancelled)?;
            match writer.write(chunk) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero).into()),
                Ok(count) => chunk = &chunk[count..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}
