//! Deterministic dense global ID assignment.

use fpas_bytecode::GlobalId;
use fpas_unit::object::{DefinitionTarget, RelocatableObject};

use crate::RegisterLinkError;
use crate::symbols::SymbolTable;

pub(super) struct GlobalIds {
    pub maps: Vec<Vec<Option<GlobalId>>>,
    pub order: Vec<(usize, usize)>,
}

pub(super) fn assign(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
) -> Result<GlobalIds, RegisterLinkError> {
    let mut maps = objects
        .iter()
        .map(|object| vec![None; object.globals.len()])
        .collect::<Vec<_>>();
    let mut order = Vec::new();
    for (object_index, object) in objects.iter().enumerate() {
        let mut local = (0..object.globals.len()).collect::<Vec<_>>();
        local.sort_by_key(|index| {
            symbols.canonical_target_name(
                objects,
                object_index,
                DefinitionTarget::Global(u32::try_from(*index).unwrap_or(u32::MAX)),
                &object.globals[*index].name,
            )
        });
        order.extend(local.into_iter().map(|index| (object_index, index)));
    }
    for (index, (object, local)) in order.iter().copied().enumerate() {
        let id = GlobalId::try_from_index(index)
            .map_err(|_| RegisterLinkError::Overflow("global IDs"))?;
        maps[object][local] = Some(id);
    }
    Ok(GlobalIds { maps, order })
}
