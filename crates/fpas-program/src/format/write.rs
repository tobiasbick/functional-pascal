//! Deterministic `.fpascp` encoder.

use crate::ProgramImage;

use super::{FormatError, executable, header};

/// Encode one verified register program image into canonical little-endian bytes.
///
/// # Errors
///
/// Returns a structured format or resource-limit error.
pub fn encode(image: &ProgramImage) -> Result<Vec<u8>, FormatError> {
    image.validate()?;
    let payload = executable::encode(image.executable().executable())?;
    header::encode(image.identity(), image.source_hashes(), &payload)
}
