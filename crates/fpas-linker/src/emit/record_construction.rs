//! Coalesced record aliases and declaration-bound defaults after source relocation.
//! See `docs/pascal/language/types/records.md` and `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{RecordConstructionInfo, RecordTypeAlias, SourceId, SourceMap};
use fpas_unit::object::RelocatableObject;

use super::strings::StringInterner;
use crate::LinkError;
use crate::plan::LinkIds;

/// Merges source-visible aliases of each coalesced nominal record layout.
pub(super) fn merge(
    objects: &[&RelocatableObject],
    ids: &LinkIds,
    sources: &SourceMap,
    strings: &mut StringInterner,
    records: &mut [fpas_bytecode::RecordLayout],
) -> Result<(), LinkError> {
    for (final_index, record) in records.iter_mut().enumerate() {
        let mut merged: Option<RecordConstructionInfo> = None;
        for (object_index, object) in objects.iter().enumerate() {
            for (local, candidate) in object.records.iter().enumerate() {
                if ids.layouts.records[object_index][local]
                    .is_none_or(|id| id.get() as usize != final_index)
                {
                    continue;
                }
                let Some(info) = &candidate.construction else {
                    continue;
                };
                if merged.is_none() {
                    merged = Some(RecordConstructionInfo {
                        owner_unit: info
                            .owner_unit
                            .as_deref()
                            .map(|name| strings.intern(name))
                            .transpose()?,
                        requires_owner: info.requires_owner,
                        defaults: info
                            .defaults
                            .iter()
                            .map(|name| {
                                name.as_deref().map(|name| strings.intern(name)).transpose()
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        aliases: Vec::new(),
                    });
                }
                let Some(merged) = merged.as_mut() else {
                    continue;
                };
                for alias in &info.aliases {
                    let path = object
                        .sources
                        .get(alias.source as usize)
                        .ok_or(LinkError::Overflow("record alias source"))?;
                    let path_id = strings.intern(path)?;
                    let source = sources
                        .sources
                        .iter()
                        .position(|source| *source == path_id)
                        .ok_or(LinkError::Overflow("record alias source relocation"))?;
                    let alias = RecordTypeAlias {
                        source: SourceId::try_from_index(source)
                            .map_err(|_| LinkError::Overflow("record alias source ID"))?,
                        name: strings.intern(&alias.name)?,
                        unit: strings.intern(&alias.unit)?,
                    };
                    if !merged.aliases.contains(&alias) {
                        merged.aliases.push(alias);
                    }
                }
            }
        }
        record.construction = merged;
    }
    Ok(())
}
