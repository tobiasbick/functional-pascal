//! Bounded `.fpascp` decoder.

use crate::image::decode_payload;
use crate::image::resources::MAX_TOTAL_STRING_BYTES;
use crate::{Digest, LinkedUnitIdentity, ProgramIdentity, ProgramImage};

use super::{
    FormatError, MAGIC, MAX_PAYLOAD_BYTES, MAX_SOURCE_PATHS, MAX_STRING_BYTES, MAX_UNITS,
    PROGRAM_FORMAT_VERSION, check_size,
};

/// Decode and validate one complete `.fpascp` byte sequence.
pub fn decode(bytes: &[u8]) -> Result<ProgramImage, FormatError> {
    let mut reader = Reader::new(bytes);
    if reader.take(MAGIC.len(), "magic")? != MAGIC {
        return Err(FormatError::InvalidMagic);
    }
    let version = reader.u16("format_version")?;
    if version != PROGRAM_FORMAT_VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }
    let bytecode_version = reader.u32("bytecode_version")?;
    let compiler_version = reader.string("compiler_version")?;
    let source_hash = reader.digest("source_hash")?;
    let options_hash = reader.digest("options_hash")?;
    let unit_count = reader.u32("unit_count")? as usize;
    check_size("units", unit_count, MAX_UNITS)?;
    let mut units = Vec::with_capacity(unit_count);
    for _ in 0..unit_count {
        units.push(LinkedUnitIdentity {
            unit_name: reader.string("unit.unit_name")?,
            object_hash: reader.digest("unit.object_hash")?,
        });
    }
    let source_path_count = reader.u32("source_path_count")? as usize;
    check_size("source_paths", source_path_count, MAX_SOURCE_PATHS)?;
    let mut source_paths = Vec::with_capacity(source_path_count);
    for _ in 0..source_path_count {
        source_paths.push(reader.string("source_path")?);
    }
    let payload_hash = reader.digest("payload_hash")?;
    let payload = reader.bytes("payload", MAX_PAYLOAD_BYTES)?;
    if reader.remaining() != 0 {
        return Err(FormatError::TrailingBytes(reader.remaining()));
    }
    if payload_hash != Digest::of(payload) {
        return Err(FormatError::PayloadHashMismatch);
    }
    let identity = ProgramIdentity {
        compiler_version,
        bytecode_version,
        source_hash,
        options_hash,
        units,
    };
    decode_payload(identity, source_paths, payload, reader.string_bytes).map_err(FormatError::Image)
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    string_bytes: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            string_bytes: 0,
        }
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
        let string_bytes =
            self.string_bytes
                .checked_add(length)
                .ok_or(FormatError::LimitExceeded {
                    field: "strings",
                    size: usize::MAX,
                    maximum: MAX_TOTAL_STRING_BYTES,
                })?;
        check_size("strings", string_bytes, MAX_TOTAL_STRING_BYTES)?;
        let bytes = self.take(length, field)?;
        let value = std::str::from_utf8(bytes).map_err(|_| FormatError::InvalidUtf8(field))?;
        self.string_bytes = string_bytes;
        Ok(value.to_string())
    }

    fn bytes(&mut self, field: &'static str, maximum: usize) -> Result<&'a [u8], FormatError> {
        let length = self.u32(field)? as usize;
        check_size(field, length, maximum)?;
        self.take(length, field)
    }
}
