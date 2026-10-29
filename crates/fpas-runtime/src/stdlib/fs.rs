//! `Std.Io.Fs` runtime implementation.
//!
//! Native path operations shared by synchronous and suspended calls.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

mod append;
mod binary;
mod directory;
pub(crate) mod failure;
mod glob;
mod io;
mod metadata;
pub(crate) mod native;
mod publication;
mod read;
pub(crate) mod results;
mod temp;
mod tree;

use std::fs;

use crate::{Intrinsic, SourceLocation};

use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::IntrinsicCall;

/// Read a bounded UTF-8 file through the same implementation as Std.Io.Fs.ReadText.
pub fn read_text_for_host(path: &str) -> std::io::Result<String> {
    read_text_for_host_controlled(path, &mut || false)
}
pub(crate) fn read_text_for_host_controlled(
    path: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> std::io::Result<String> {
    read::read_text_controlled(path, cancelled).map_err(|error| match error {
        failure::Failure::Io(error) => error,
        failure::Failure::LimitExceeded(_, _) => {
            std::io::Error::from(std::io::ErrorKind::FileTooLarge)
        }
        _ => std::io::Error::from(std::io::ErrorKind::InvalidData),
    })
}
/// Write a UTF-8 file through the same implementation as Std.Io.Fs.WriteText.
pub fn write_text_for_host(path: &str, text: &str) -> std::io::Result<()> {
    fs::write(path, text)
}
pub(crate) fn write_text_for_host_controlled(
    path: &str,
    text: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> std::io::Result<()> {
    io::check(cancelled).map_err(|_| std::io::Error::from(std::io::ErrorKind::Interrupted))?;
    let mut file = fs::File::create(path)?;
    io::write_all(&mut file, text.as_bytes(), cancelled).map_err(|error| match error {
        failure::Failure::Io(error) => error,
        _ => std::io::Error::from(std::io::ErrorKind::Interrupted),
    })
}

/// Execute a `Std.Io.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Fs(operation) = intrinsic else {
        return Ok(None);
    };
    if !native::supported(operation) {
        return Ok(None);
    }
    let outcome = match native::Call::prepare(operation, call, location)? {
        Ok(request) => request.run(|| false),
        Err(error) => native::Outcome::invalid(operation, error),
    };
    let value = outcome.into_value(call.factory(), location)?;
    call.push(value);
    Ok(Some(()))
}

#[cfg(test)]
mod binary_tests;

#[cfg(test)]
mod tests;
