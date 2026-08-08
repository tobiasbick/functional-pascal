//! Deterministic type IDs, declaration-order fields, and enum variants.

use fpas_bytecode::{EnumTypeId, EnumVariantId, RecordTypeId};
use fpas_unit::object::{DefinitionTarget, RelocatableObject};

use crate::RegisterLinkError;
use crate::symbols::SymbolTable;

pub(super) struct LayoutIds {
    pub records: Vec<Vec<Option<RecordTypeId>>>,
    pub record_order: Vec<(usize, usize)>,
    pub enums: Vec<Vec<Option<EnumTypeId>>>,
    pub enum_order: Vec<(usize, usize)>,
    pub variants: Vec<Vec<Vec<Option<EnumVariantId>>>>,
}

pub(super) fn assign(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
) -> Result<LayoutIds, RegisterLinkError> {
    let mut records = objects
        .iter()
        .map(|object| vec![None; object.records.len()])
        .collect::<Vec<_>>();
    let mut record_order = Vec::new();
    let mut enums = objects
        .iter()
        .map(|object| vec![None; object.enums.len()])
        .collect::<Vec<_>>();
    let mut enum_order = Vec::new();
    for (object_index, object) in objects.iter().enumerate() {
        let mut local_records = (0..object.records.len()).collect::<Vec<_>>();
        local_records.sort_by_key(|index| {
            symbols.canonical_target_name(
                objects,
                object_index,
                DefinitionTarget::Record(u32::try_from(*index).unwrap_or(u32::MAX)),
                &object.records[*index].name,
            )
        });
        record_order.extend(local_records.into_iter().map(|index| (object_index, index)));

        let mut local_enums = (0..object.enums.len()).collect::<Vec<_>>();
        local_enums.sort_by_key(|index| {
            symbols.canonical_target_name(
                objects,
                object_index,
                DefinitionTarget::Enum(u32::try_from(*index).unwrap_or(u32::MAX)),
                &object.enums[*index].name,
            )
        });
        enum_order.extend(local_enums.into_iter().map(|index| (object_index, index)));
    }
    for (index, (object, local)) in record_order.iter().copied().enumerate() {
        records[object][local] = Some(
            RecordTypeId::try_from_index(index)
                .map_err(|_| RegisterLinkError::Overflow("record type IDs"))?,
        );
    }
    for (index, (object, local)) in enum_order.iter().copied().enumerate() {
        enums[object][local] = Some(
            EnumTypeId::try_from_index(index)
                .map_err(|_| RegisterLinkError::Overflow("enum type IDs"))?,
        );
    }
    let mut variants = objects
        .iter()
        .map(|object| {
            object
                .enums
                .iter()
                .map(|layout| vec![None; layout.variants.len()])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut next = 0;
    for (object, enumeration) in &enum_order {
        for variant in 0..objects[*object].enums[*enumeration].variants.len() {
            variants[*object][*enumeration][variant] = Some(
                EnumVariantId::try_from_index(next)
                    .map_err(|_| RegisterLinkError::Overflow("enum variant IDs"))?,
            );
            next += 1;
        }
    }
    Ok(LayoutIds {
        records,
        record_order,
        enums,
        enum_order,
        variants,
    })
}
