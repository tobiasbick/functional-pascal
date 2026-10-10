//! Source-visible distinct type names after source and debug-type relocation.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use fpas_bytecode::{DistinctTypeName, SourceId, SourceMap};
use fpas_unit::object::RelocatableObject;

use super::strings::StringInterner;
use crate::LinkError;
use crate::plan::DebugTypeIds;

/// Relocates every object's distinct names into final source and debug-type coordinates.
pub(super) fn merge(
    objects: &[&RelocatableObject],
    debug_type_ids: &DebugTypeIds,
    sources: &SourceMap,
    strings: &mut StringInterner,
) -> Result<Vec<DistinctTypeName>, LinkError> {
    let mut merged = Vec::new();
    for (object_index, object) in objects.iter().enumerate() {
        for distinct in &object.distinct_types {
            let path = object
                .sources
                .get(distinct.source as usize)
                .ok_or(LinkError::Overflow("distinct type source"))?;
            let path_id = strings.intern(path)?;
            let source = sources
                .sources
                .iter()
                .position(|source| *source == path_id)
                .ok_or(LinkError::Overflow("distinct type source relocation"))?;
            let name = DistinctTypeName {
                source: SourceId::try_from_index(source)
                    .map_err(|_| LinkError::Overflow("distinct type source ID"))?,
                name: strings.intern(&distinct.name)?,
                underlying: debug_type_ids.translate(object_index, distinct.underlying)?,
            };
            if !merged.contains(&name) {
                merged.push(name);
            }
        }
    }
    Ok(merged)
}
