//! Bounded `.fpascp` decoder.

use crate::ProgramImage;

use super::{FormatError, executable, header};

/// Decode and verify one complete portable register program image.
///
/// # Errors
///
/// Returns a structured format, resource, identity, or executable-verifier failure.
pub fn decode(bytes: &[u8]) -> Result<ProgramImage, FormatError> {
    let decoded = header::decode(bytes)?;
    let executable = executable::decode(decoded.payload)?;
    ProgramImage::from_decoded(decoded.identity, decoded.source_hashes, executable)
        .map_err(FormatError::Image)
}
