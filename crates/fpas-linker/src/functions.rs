//! Deterministic function ID assignment.

use fpas_bytecode::FunctionId;
use fpas_unit::object::{DefinitionTarget, RelocatableObject};

use crate::LinkError;
use crate::symbols::SymbolTable;

pub(super) struct FunctionIds {
    pub maps: Vec<Vec<Option<FunctionId>>>,
    pub order: Vec<(usize, usize)>,
}

pub(super) fn assign(
    objects: &[&RelocatableObject],
    program_index: usize,
    entry: usize,
    symbols: &SymbolTable,
) -> Result<FunctionIds, LinkError> {
    let mut maps = objects
        .iter()
        .map(|object| vec![None; object.functions.len()])
        .collect::<Vec<_>>();
    let mut order = vec![(program_index, entry)];
    for (object_index, object) in objects.iter().enumerate() {
        let mut local = (0..object.functions.len())
            .filter(|index| (object_index, *index) != (program_index, entry))
            .collect::<Vec<_>>();
        local.sort_by_key(|index| {
            symbols.canonical_target_name(
                objects,
                object_index,
                DefinitionTarget::Function(u32::try_from(*index).unwrap_or(u32::MAX)),
                &object.functions[*index].name,
            )
        });
        order.extend(local.into_iter().map(|index| (object_index, index)));
    }
    for (index, (object, local)) in order.iter().copied().enumerate() {
        let id =
            FunctionId::try_from_index(index).map_err(|_| LinkError::Overflow("function IDs"))?;
        maps[object][local] = Some(id);
    }
    Ok(FunctionIds { maps, order })
}
