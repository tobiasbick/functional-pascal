//! Deterministic `.fpascp` encoder.

use crate::image::encode_payload;
use crate::{Digest, ProgramImage};

use super::{
    FormatError, MAGIC, MAX_PAYLOAD_BYTES, PROGRAM_FORMAT_VERSION, check_size, validate_for_write,
};

/// Encode one complete program image into deterministic `.fpascp` bytes.
pub fn encode(image: &ProgramImage) -> Result<Vec<u8>, FormatError> {
    validate_for_write(image)?;
    let payload = encode_payload(image)?;
    check_size("payload", payload.len(), MAX_PAYLOAD_BYTES)?;

    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    write_u16(&mut output, PROGRAM_FORMAT_VERSION);
    write_u32(&mut output, image.identity().bytecode_version);
    write_string(&mut output, &image.identity().compiler_version)?;
    write_digest(&mut output, image.identity().source_hash);
    write_digest(&mut output, image.identity().options_hash);
    write_u32(
        &mut output,
        u32::try_from(image.identity().units.len()).map_err(|_| FormatError::LimitExceeded {
            field: "units",
            size: image.identity().units.len(),
            maximum: u32::MAX as usize,
        })?,
    );
    for unit in &image.identity().units {
        write_string(&mut output, &unit.unit_name)?;
        write_digest(&mut output, unit.object_hash);
    }
    write_u32(
        &mut output,
        u32::try_from(image.source_paths().len()).map_err(|_| FormatError::LimitExceeded {
            field: "source_paths",
            size: image.source_paths().len(),
            maximum: u32::MAX as usize,
        })?,
    );
    for source_path in image.source_paths() {
        write_string(&mut output, source_path)?;
    }
    write_digest(&mut output, Digest::of(&payload));
    write_bytes(&mut output, &payload)?;
    Ok(output)
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_digest(output: &mut Vec<u8>, digest: Digest) {
    output.extend_from_slice(digest.as_bytes());
}

fn write_string(output: &mut Vec<u8>, value: &str) -> Result<(), FormatError> {
    let length = u32::try_from(value.len()).map_err(|_| FormatError::LimitExceeded {
        field: "string",
        size: value.len(),
        maximum: u32::MAX as usize,
    })?;
    write_u32(output, length);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) -> Result<(), FormatError> {
    let length = u32::try_from(value.len()).map_err(|_| FormatError::LimitExceeded {
        field: "payload",
        size: value.len(),
        maximum: u32::MAX as usize,
    })?;
    write_u32(output, length);
    output.extend_from_slice(value);
    Ok(())
}
