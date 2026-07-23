//! Deterministic `.fpascu` encoder.

use crate::{CompiledUnit, Digest};

use super::{FORMAT_VERSION, FormatError, MAGIC, validate_for_write};

/// Encodes one compiled unit into deterministic `.fpascu` bytes.
pub fn encode(unit: &CompiledUnit) -> Result<Vec<u8>, FormatError> {
    validate_for_write(unit)?;
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    write_u16(&mut output, FORMAT_VERSION);
    write_u32(&mut output, unit.identity.bytecode_version);
    write_string(&mut output, &unit.identity.compiler_version)?;
    write_string(&mut output, &unit.identity.unit_name)?;
    write_digest(&mut output, unit.identity.source_hash);
    write_digest(&mut output, unit.identity.interface_hash);
    write_digest(&mut output, unit.identity.object_hash);
    write_digest(&mut output, unit.identity.options_hash);
    write_u32(
        &mut output,
        u32::try_from(unit.identity.dependencies.len()).map_err(|_| {
            FormatError::LimitExceeded {
                field: "dependencies",
                size: unit.identity.dependencies.len(),
                maximum: u32::MAX as usize,
            }
        })?,
    );
    for dependency in &unit.identity.dependencies {
        write_string(&mut output, &dependency.unit_name)?;
        write_digest(&mut output, dependency.interface_hash);
    }
    write_bytes(&mut output, &unit.interface)?;
    write_bytes(&mut output, &unit.object)?;
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
