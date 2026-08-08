//! Canonical executable section directory construction and validation.

use super::{FormatError, check_limit, checked_u32};

pub(super) const TAGS: [u16; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const DIRECTORY_PREFIX_BYTES: usize = 4;
const DIRECTORY_ENTRY_BYTES: usize = 16;

pub(super) struct EncodedSection {
    pub(super) tag: u16,
    pub(super) item_count: usize,
    pub(super) bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(super) struct DecodedSection<'a> {
    pub(super) tag: u16,
    pub(super) item_count: usize,
    pub(super) bytes: &'a [u8],
}

pub(super) fn encode(sections: Vec<EncodedSection>) -> Result<Vec<u8>, FormatError> {
    if sections.len() != TAGS.len() {
        return Err(FormatError::SectionCount {
            actual: sections.len(),
            expected: TAGS.len(),
        });
    }
    let directory_bytes = directory_bytes(sections.len())?;
    let payload_bytes = sections
        .iter()
        .try_fold(directory_bytes, |total, section| {
            total
                .checked_add(section.bytes.len())
                .ok_or(FormatError::LimitExceeded {
                    field: "payload",
                    size: usize::MAX,
                    maximum: fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
                })
        })?;
    check_limit(
        "payload",
        payload_bytes,
        fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
    )?;
    let mut output = Vec::with_capacity(payload_bytes);
    write_u16(&mut output, sections.len() as u16);
    write_u16(&mut output, 0);
    let mut offset = directory_bytes;
    for (index, section) in sections.iter().enumerate() {
        let expected = TAGS[index];
        if section.tag != expected {
            return Err(FormatError::SectionTag {
                index,
                actual: section.tag,
                expected,
            });
        }
        write_u16(&mut output, section.tag);
        write_u16(&mut output, 0);
        write_u32(&mut output, checked_u32("section_offset", offset)?);
        write_u32(
            &mut output,
            checked_u32("section_length", section.bytes.len())?,
        );
        write_u32(
            &mut output,
            checked_u32("section_item_count", section.item_count)?,
        );
        offset = offset
            .checked_add(section.bytes.len())
            .ok_or(FormatError::LimitExceeded {
                field: "payload",
                size: usize::MAX,
                maximum: fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
            })?;
    }
    for section in sections {
        output.extend_from_slice(&section.bytes);
    }
    Ok(output)
}

pub(super) fn decode(payload: &[u8]) -> Result<Vec<DecodedSection<'_>>, FormatError> {
    check_limit(
        "payload",
        payload.len(),
        fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
    )?;
    let mut reader = SectionReader::new(payload, "section_directory");
    let count = reader.u16("section_count")? as usize;
    if count != TAGS.len() {
        return Err(FormatError::SectionCount {
            actual: count,
            expected: TAGS.len(),
        });
    }
    if count > fpas_bytecode::limits::MAX_SECTIONS {
        return Err(FormatError::SectionItemCount {
            tag: 0,
            count,
            maximum: fpas_bytecode::limits::MAX_SECTIONS,
        });
    }
    let flags = reader.u16("section_directory_flags")?;
    if flags != 0 {
        return Err(FormatError::UnsupportedFlags {
            field: "section_directory",
            flags,
        });
    }
    let expected_start = directory_bytes(count)?;
    let mut ranges = Vec::with_capacity(count);
    let mut expected_offset = expected_start;
    for (index, expected_tag) in TAGS.iter().copied().enumerate() {
        let tag = reader.u16("section_tag")?;
        if tag != expected_tag {
            return Err(FormatError::SectionTag {
                index,
                actual: tag,
                expected: expected_tag,
            });
        }
        let flags = reader.u16("section_flags")?;
        if flags != 0 {
            return Err(FormatError::UnsupportedFlags {
                field: "section",
                flags,
            });
        }
        let offset = reader.u32("section_offset")? as usize;
        let length = reader.u32("section_length")? as usize;
        let item_count = reader.u32("section_item_count")? as usize;
        let end = offset.checked_add(length);
        if offset != expected_offset || end.is_none_or(|end| end > payload.len()) {
            return Err(FormatError::SectionRange {
                tag,
                offset,
                expected_offset,
                length,
                payload: payload.len(),
            });
        }
        expected_offset = end.unwrap_or(payload.len());
        ranges.push((tag, offset, length, item_count));
    }
    if expected_offset != payload.len() {
        return Err(FormatError::TrailingBytes {
            container: "section payload",
            count: payload.len().saturating_sub(expected_offset),
        });
    }
    ranges
        .into_iter()
        .map(|(tag, offset, length, item_count)| {
            let bytes = payload
                .get(offset..offset + length)
                .ok_or(FormatError::SectionRange {
                    tag,
                    offset,
                    expected_offset: offset,
                    length,
                    payload: payload.len(),
                })?;
            Ok(DecodedSection {
                tag,
                item_count,
                bytes,
            })
        })
        .collect()
}

fn directory_bytes(count: usize) -> Result<usize, FormatError> {
    let entries = count
        .checked_mul(DIRECTORY_ENTRY_BYTES)
        .ok_or(FormatError::LimitExceeded {
            field: "section_directory",
            size: usize::MAX,
            maximum: fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
        })?;
    DIRECTORY_PREFIX_BYTES
        .checked_add(entries)
        .ok_or(FormatError::LimitExceeded {
            field: "section_directory",
            size: usize::MAX,
            maximum: fpas_bytecode::limits::MAX_PAYLOAD_BYTES,
        })
}

pub(super) struct SectionReader<'a> {
    bytes: &'a [u8],
    position: usize,
    container: &'static str,
}

impl<'a> SectionReader<'a> {
    pub(super) const fn new(bytes: &'a [u8], container: &'static str) -> Self {
        Self {
            bytes,
            position: 0,
            container,
        }
    }

    pub(super) fn finish(self) -> Result<(), FormatError> {
        let remaining = self.bytes.len().saturating_sub(self.position);
        if remaining != 0 {
            return Err(FormatError::TrailingBytes {
                container: self.container,
                count: remaining,
            });
        }
        Ok(())
    }

    pub(super) fn take(
        &mut self,
        length: usize,
        field: &'static str,
    ) -> Result<&'a [u8], FormatError> {
        let Some(end) = self.position.checked_add(length) else {
            return Err(FormatError::LimitExceeded {
                field,
                size: length,
                maximum: self.bytes.len().saturating_sub(self.position),
            });
        };
        let Some(value) = self.bytes.get(self.position..end) else {
            return Err(FormatError::Truncated(field));
        };
        self.position = end;
        Ok(value)
    }

    pub(super) fn u8(&mut self, field: &'static str) -> Result<u8, FormatError> {
        Ok(self.take(1, field)?[0])
    }

    pub(super) fn u16(&mut self, field: &'static str) -> Result<u16, FormatError> {
        let bytes = self.take(2, field)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(super) fn u32(&mut self, field: &'static str) -> Result<u32, FormatError> {
        let bytes = self.take(4, field)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn u64(&mut self, field: &'static str) -> Result<u64, FormatError> {
        let bytes = self.take(8, field)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub(super) fn i64(&mut self, field: &'static str) -> Result<i64, FormatError> {
        let bytes = self.take(8, field)?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

pub(super) fn write_u8(output: &mut Vec<u8>, value: u8) {
    output.push(value);
}

pub(super) fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i64(output: &mut Vec<u8>, value: i64) {
    output.extend_from_slice(&value.to_le_bytes());
}
