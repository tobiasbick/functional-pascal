//! Canonical executable section directory construction and validation.

use std::ops::{Deref, DerefMut};

use fpas_binary::ByteReader;
pub(super) use fpas_binary::{write_i64, write_u8, write_u16, write_u32, write_u64};

use super::{FormatError, check_limit, checked_u32};

pub(super) const TAGS: [u16; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const _: () = assert!(TAGS.len() <= fpas_bytecode::limits::MAX_SECTIONS);
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

/// Decode the bounded section directory and validate contiguous payload ranges.
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
        let Some(end) = offset
            .checked_add(length)
            .filter(|end| *end <= payload.len())
        else {
            return Err(FormatError::SectionRange {
                tag,
                offset,
                expected_offset,
                length,
                payload: payload.len(),
            });
        };
        if offset != expected_offset {
            return Err(FormatError::SectionRange {
                tag,
                offset,
                expected_offset,
                length,
                payload: payload.len(),
            });
        }
        expected_offset = end;
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

/// Bounds-checked reader for one section that rejects unread trailing bytes.
pub(super) struct SectionReader<'a> {
    reader: ByteReader<'a, FormatError>,
    container: &'static str,
}

impl<'a> SectionReader<'a> {
    pub(super) const fn new(bytes: &'a [u8], container: &'static str) -> Self {
        Self {
            reader: ByteReader::new(bytes),
            container,
        }
    }

    pub(super) fn finish(self) -> Result<(), FormatError> {
        let remaining = self.reader.remaining();
        if remaining != 0 {
            return Err(FormatError::TrailingBytes {
                container: self.container,
                count: remaining,
            });
        }
        Ok(())
    }

    /// Reject an entry count that cannot fit in the unread section bytes.
    pub(super) fn ensure_entries(
        &self,
        count: usize,
        minimum_bytes: usize,
        field: &'static str,
    ) -> Result<(), FormatError> {
        if count > self.reader.remaining() / minimum_bytes {
            return Err(FormatError::Truncated(field));
        }
        Ok(())
    }
}

impl<'a> Deref for SectionReader<'a> {
    type Target = ByteReader<'a, FormatError>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl DerefMut for SectionReader<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
