//! Bounded `.fpascu` decoder.

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
    let mut reader = Reader::new(bytes);
    if reader.take(MAGIC.len(), "magic")? != MAGIC {
        return Err(FormatError::InvalidMagic);
    }
    let version = reader.u16("format_version")?;
    if version != FORMAT_VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }
    let bytecode_version = reader.u32("bytecode_version")?;
    let compiler_version = reader.string("compiler_version")?;
    let unit_name = reader.string("unit_name")?;
    let source_hash = reader.digest("source_hash")?;
    let interface_hash = reader.digest("interface_hash")?;
    let object_hash = reader.digest("object_hash")?;
    let options_hash = reader.digest("options_hash")?;
    let dependency_count = reader.u32("dependency_count")? as usize;
    check_size("dependencies", dependency_count, MAX_DEPENDENCIES)?;
    let mut dependencies = Vec::with_capacity(dependency_count);
    for _ in 0..dependency_count {
        dependencies.push(DependencyIdentity {
            unit_name: reader.string("dependency.unit_name")?,
            interface_hash: reader.digest("dependency.interface_hash")?,
        });
    }
    let interface = reader.bytes("interface", MAX_PAYLOAD_BYTES)?;
    let object = reader.bytes("object", MAX_PAYLOAD_BYTES)?;
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

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    fn take(&mut self, length: usize, field: &'static str) -> Result<&'a [u8], FormatError> {
        let Some(end) = self.position.checked_add(length) else {
            return Err(FormatError::LimitExceeded {
                field,
                size: length,
                maximum: self.remaining(),
            });
        };
        let Some(value) = self.bytes.get(self.position..end) else {
            return Err(FormatError::Truncated(field));
        };
        self.position = end;
        Ok(value)
    }

    fn u16(&mut self, field: &'static str) -> Result<u16, FormatError> {
        let bytes = self.take(2, field)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, FormatError> {
        let bytes = self.take(4, field)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn digest(&mut self, field: &'static str) -> Result<Digest, FormatError> {
        let bytes = self.take(Digest::LENGTH, field)?;
        let mut digest = [0_u8; Digest::LENGTH];
        digest.copy_from_slice(bytes);
        Ok(Digest::from_bytes(digest))
    }

    fn string(&mut self, field: &'static str) -> Result<String, FormatError> {
        let length = self.u32(field)? as usize;
        check_size(field, length, MAX_STRING_BYTES)?;
        let bytes = self.take(length, field)?;
        let value = std::str::from_utf8(bytes).map_err(|_| FormatError::InvalidUtf8(field))?;
        Ok(value.to_string())
    }

    fn bytes(&mut self, field: &'static str, maximum: usize) -> Result<Vec<u8>, FormatError> {
        let length = self.u32(field)? as usize;
        check_size(field, length, maximum)?;
        Ok(self.take(length, field)?.to_vec())
    }
}
