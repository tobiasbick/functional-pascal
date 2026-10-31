//! Deterministic `.fpascp` encoder.

use crate::{Digest, ProgramImage};

use super::{FormatError, executable, header};

/// Encode one verified register program image into canonical little-endian bytes.
///
/// # Errors
///
/// Returns a structured format or resource-limit error.
pub fn encode(image: &ProgramImage) -> Result<Vec<u8>, FormatError> {
    let payload = executable::encode(image.executable().executable())?;
    header::encode(image.identity(), image.source_hashes(), &payload)
}

/// BLAKE3 digest of an executable's canonical payload encoding, without image headers.
///
/// Equal executables, including their debug metadata, have equal digests regardless of
/// whether they were compiled from source or decoded from an image.
///
/// # Errors
///
/// Returns a structured format or resource-limit error.
pub fn executable_digest(executable: &fpas_code::Executable) -> Result<Digest, FormatError> {
    executable::encode(executable).map(Digest::of)
}
