//! Bounded `.fpascu` decoder.

use fpas_binary::ByteReader;

use crate::{CompiledUnit, DependencyIdentity, Digest, UnitIdentity};

use super::{
    FORMAT_VERSION, FormatError, MAGIC, MAX_DEPENDENCIES, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    check_sidecar_size, check_size,
};

/// Decodes and validates one complete `.fpascu` byte sequence.
///
/// # Errors
///
/// Returns [`FormatError`] when the envelope exceeds its resource budget, is
/// malformed or incompatible, or fails payload integrity validation.
pub fn decode(bytes: &[u8]) -> Result<CompiledUnit, FormatError> {
    check_sidecar_size(bytes.len())?;
    let mut reader = ByteReader::<FormatError>::new(bytes);
    if reader.take(MAGIC.len(), "magic")? != MAGIC {
        return Err(FormatError::InvalidMagic);
    }
    let version = reader.u16("format_version")?;
    if version != FORMAT_VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }
    let bytecode_version = reader.u32("bytecode_version")?;
    let compiler_version = reader.string("compiler_version", MAX_STRING_BYTES)?;
    let unit_name = reader.string("unit_name", MAX_STRING_BYTES)?;
    let source_hash = reader.digest("source_hash")?;
    let interface_hash = reader.digest("interface_hash")?;
    let object_hash = reader.digest("object_hash")?;
    let options_hash = reader.digest("options_hash")?;
    let dependency_count = reader.u32("dependency_count")? as usize;
    check_size("dependencies", dependency_count, MAX_DEPENDENCIES)?;
    let mut dependencies = Vec::with_capacity(dependency_count);
    for _ in 0..dependency_count {
        dependencies.push(DependencyIdentity {
            unit_name: reader.string("dependency.unit_name", MAX_STRING_BYTES)?,
            interface_hash: reader.digest("dependency.interface_hash")?,
        });
    }
    let interface = reader.bytes("interface", MAX_PAYLOAD_BYTES)?.to_vec();
    let object = reader.bytes("object", MAX_PAYLOAD_BYTES)?.to_vec();
    if reader.remaining() != 0 {
        return Err(FormatError::TrailingBytes(reader.remaining()));
    }
    if interface_hash != Digest::of(&interface) {
        return Err(FormatError::InterfaceHashMismatch);
    }
    if object_hash != Digest::of(&object) {
        return Err(FormatError::ObjectHashMismatch);
    }
    Ok(CompiledUnit {
        identity: UnitIdentity {
            unit_name,
            source_hash,
            interface_hash,
            object_hash,
            compiler_version,
            bytecode_version,
            options_hash,
            dependencies,
        },
        interface,
        object,
    })
}
