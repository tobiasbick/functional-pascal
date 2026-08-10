//! Fixed portable program envelope and build identity codec.

use crate::{Digest, LinkedUnitIdentity, ProgramIdentity};

use super::{FormatError, PROGRAM_FORMAT_VERSION, check_limit, checked_u32};

const MAGIC: &[u8; 8] = b"FPASCP\0\0";
const FLAGS: u16 = 0;

pub(super) struct DecodedHeader<'a> {
    pub(super) identity: ProgramIdentity,
    pub(super) source_hashes: Vec<Digest>,
    pub(super) payload: &'a [u8],
}

pub(super) fn encode(
    identity: &ProgramIdentity,
    source_hashes: &[Digest],
    payload: &[u8],
) -> Result<Vec<u8>, FormatError> {
    check_limit(
        "payload",
        payload.len(),
        fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
    )?;
    check_limit(
        "linked_units",
        identity.units.len(),
        fpas_bytecode::limits::MAX_LINKED_UNITS,
    )?;
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    write_u16(&mut output, PROGRAM_FORMAT_VERSION);
    write_u32(&mut output, identity.bytecode_version);
    write_u16(&mut output, FLAGS);
    write_string(&mut output, "compiler_version", &identity.compiler_version)?;
    write_digest(&mut output, identity.source_hash);
    write_digest(&mut output, identity.options_hash);
    write_u32(
        &mut output,
        checked_u32("linked_units", identity.units.len())?,
    );
    for unit in &identity.units {
        write_string(&mut output, "unit_name", &unit.unit_name)?;
        write_digest(&mut output, unit.object_hash);
    }
    write_u32(
        &mut output,
        checked_u32("source_hashes", source_hashes.len())?,
    );
    for hash in source_hashes {
        write_digest(&mut output, *hash);
    }
    write_u32(&mut output, checked_u32("payload", payload.len())?);
    write_digest(&mut output, Digest::of(payload));
    output.extend_from_slice(payload);
    Ok(output)
}

pub(super) fn decode(bytes: &[u8]) -> Result<DecodedHeader<'_>, FormatError> {
    let mut reader = Reader::new(bytes);
    if reader.take(MAGIC.len(), "magic")? != MAGIC {
        return Err(FormatError::InvalidMagic);
    }
    let format_version = reader.u16("program_format_version")?;
    if format_version != PROGRAM_FORMAT_VERSION {
        return Err(FormatError::UnsupportedVersion {
            image: format_version,
            runtime: PROGRAM_FORMAT_VERSION,
        });
    }
    let bytecode_version = reader.u32("bytecode_version")?;
    if bytecode_version != fpas_bytecode::BYTECODE_VERSION {
        return Err(FormatError::UnsupportedBytecodeVersion {
            image: bytecode_version,
            runtime: fpas_bytecode::BYTECODE_VERSION,
        });
    }
    let flags = reader.u16("flags")?;
    if flags != FLAGS {
        return Err(FormatError::UnsupportedFlags {
            field: "header",
            flags,
        });
    }
    let compiler_version = reader.string("compiler_version")?;
    let source_hash = reader.digest("source_hash")?;
    let options_hash = reader.digest("options_hash")?;
    let unit_count = reader.u32("linked_unit_count")? as usize;
    check_limit(
        "linked_units",
        unit_count,
        fpas_bytecode::limits::MAX_LINKED_UNITS,
    )?;
    let mut units = Vec::with_capacity(unit_count);
    for _ in 0..unit_count {
        units.push(LinkedUnitIdentity {
            unit_name: reader.string("unit_name")?,
            object_hash: reader.digest("unit_object_hash")?,
        });
    }
    let source_hash_count = reader.u32("source_hash_count")? as usize;
    check_limit(
        "source_hashes",
        source_hash_count,
        fpas_bytecode::limits::MAX_SOURCE_PATHS,
    )?;
    let mut source_hashes = Vec::with_capacity(source_hash_count);
    for _ in 0..source_hash_count {
        source_hashes.push(reader.digest("source_hash")?);
    }
    let payload_len = reader.u32("payload_len")? as usize;
    check_limit(
        "payload",
        payload_len,
        fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
    )?;
    let payload_digest = reader.digest("payload_digest")?;
    let payload = reader.take(payload_len, "payload")?;
    if reader.remaining() != 0 {
        return Err(FormatError::TrailingBytes {
            container: "envelope",
            count: reader.remaining(),
        });
    }
    if payload_digest != Digest::of(payload) {
        return Err(FormatError::PayloadHashMismatch);
    }
    Ok(DecodedHeader {
        identity: ProgramIdentity {
            compiler_version,
            bytecode_version,
            source_hash,
            options_hash,
            units,
        },
        source_hashes,
        payload,
    })
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

fn write_string(output: &mut Vec<u8>, field: &'static str, value: &str) -> Result<(), FormatError> {
    check_limit(
        field,
        value.len(),
        fpas_bytecode::limits::MAX_IDENTITY_STRING_BYTES,
    )?;
    write_u32(output, checked_u32(field, value.len())?);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
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
        check_limit(
            field,
            length,
            fpas_bytecode::limits::MAX_IDENTITY_STRING_BYTES,
        )?;
        let bytes = self.take(length, field)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| FormatError::InvalidUtf8(field))
    }
}
