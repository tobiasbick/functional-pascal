//! Binary source-visible distinct type names for debugger conversions.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{DebugTypeId, DistinctTypeName, Executable, SourceId, StringId};

use super::super::FormatError;
use super::super::sections::{DecodedSection, EncodedSection, SectionReader, TAGS, write_u32};
use super::check_count;

const ENTRY_BYTES: usize = 12;

/// Encodes each name as source, name string, and underlying debug type.
pub(super) fn encode(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[11],
        executable.distinct_types.len(),
        fpas_bytecode::limits::MAX_STRINGS,
    )?;
    let mut bytes = Vec::with_capacity(executable.distinct_types.len() * ENTRY_BYTES);
    for distinct in &executable.distinct_types {
        write_u32(&mut bytes, distinct.source.get());
        write_u32(&mut bytes, distinct.name.get());
        write_u32(&mut bytes, distinct.underlying.get());
    }
    Ok(EncodedSection {
        tag: TAGS[11],
        item_count: executable.distinct_types.len(),
        bytes,
    })
}

/// Decodes bounded entries; executable verification checks every reference.
pub(super) fn decode(section: DecodedSection<'_>) -> Result<Vec<DistinctTypeName>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_STRINGS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "distinct type section");
    reader.ensure_entries(section.item_count, ENTRY_BYTES, "distinct_type")?;
    let mut names = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        names.push(DistinctTypeName {
            source: SourceId::new(reader.u32("distinct_type_source")?),
            name: StringId::new(reader.u32("distinct_type_name")?),
            underlying: DebugTypeId::new(reader.u32("distinct_type_underlying")?),
        });
    }
    reader.finish()?;
    Ok(names)
}
