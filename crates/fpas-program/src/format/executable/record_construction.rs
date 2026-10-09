//! Binary nominal construction metadata. See `docs/pascal/tools/debugger.md`.

use super::super::sections::{SectionReader, write_u8, write_u32};
use super::super::{FormatError, checked_u32};
use super::{check_count, read_bool};
use fpas_bytecode::{RecordConstructionInfo, RecordTypeAlias, SourceId, StringId};

/// Encodes visible names and declaration-order default routine references.
pub(super) fn encode(
    info: Option<&RecordConstructionInfo>,
    bytes: &mut Vec<u8>,
) -> Result<(), FormatError> {
    write_u8(bytes, u8::from(info.is_some()));
    let Some(info) = info else {
        return Ok(());
    };
    write_u32(bytes, info.owner_unit.map_or(u32::MAX, |name| name.get()));
    write_u8(bytes, u8::from(info.requires_owner));
    check_count(0, info.aliases.len(), fpas_bytecode::limits::MAX_STRINGS)?;
    write_u32(bytes, checked_u32("record_aliases", info.aliases.len())?);
    for alias in &info.aliases {
        write_u32(bytes, alias.source.get());
        write_u32(bytes, alias.name.get());
        write_u32(bytes, alias.unit.get());
    }
    check_count(
        0,
        info.defaults.len(),
        fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
    )?;
    write_u32(bytes, checked_u32("record_defaults", info.defaults.len())?);
    for default in &info.defaults {
        write_u32(bytes, default.map_or(u32::MAX, |name| name.get()));
    }
    Ok(())
}

/// Decodes bounded construction metadata without interpreting source names.
pub(super) fn decode(
    reader: &mut SectionReader<'_>,
) -> Result<Option<RecordConstructionInfo>, FormatError> {
    if !read_bool(reader, "record_construction")? {
        return Ok(None);
    }
    let owner_unit = optional_string(reader.u32("record_owner_unit")?);
    let requires_owner = read_bool(reader, "record_requires_owner")?;
    let count = reader.u32("record_alias_count")? as usize;
    check_count(0, count, fpas_bytecode::limits::MAX_STRINGS)?;
    reader.ensure_entries(count, 12, "record_alias")?;
    let mut aliases = Vec::with_capacity(count);
    for _ in 0..count {
        aliases.push(RecordTypeAlias {
            source: SourceId::new(reader.u32("record_alias_source")?),
            name: StringId::new(reader.u32("record_alias_name")?),
            unit: StringId::new(reader.u32("record_alias_unit")?),
        });
    }
    let count = reader.u32("record_default_count")? as usize;
    check_count(0, count, fpas_bytecode::limits::MAX_LAYOUT_FIELDS)?;
    reader.ensure_entries(count, 4, "record_default")?;
    let mut defaults = Vec::with_capacity(count);
    for _ in 0..count {
        defaults.push(optional_string(reader.u32("record_default")?));
    }
    Ok(Some(RecordConstructionInfo {
        owner_unit,
        requires_owner,
        aliases,
        defaults,
    }))
}

fn optional_string(id: u32) -> Option<StringId> {
    (id != u32::MAX).then(|| StringId::new(id))
}
