//! Deterministic type IDs, declaration-order fields, and enum variants.

use std::collections::BTreeMap;

use fpas_bytecode::{EnumTypeId, EnumVariantId, RecordTypeId};
use fpas_unit::object::{DefinitionTarget, RelocatableObject};

use super::SymbolTable;
use crate::LinkError;

pub(crate) struct LayoutIds {
    pub records: Vec<Vec<Option<RecordTypeId>>>,
    pub record_order: Vec<(usize, usize)>,
    pub enums: Vec<Vec<Option<EnumTypeId>>>,
    pub enum_order: Vec<(usize, usize)>,
    pub variants: Vec<Vec<Vec<Option<EnumVariantId>>>>,
}

pub(super) fn assign(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
) -> Result<LayoutIds, LinkError> {
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
    let mut canonical_records = BTreeMap::new();
    let mut canonical_enums = BTreeMap::new();
    for (object_index, object) in objects.iter().enumerate() {
        let mut local_records = (0..object.records.len()).collect::<Vec<_>>();
        local_records
            .sort_by_key(|index| canonical_record_name(objects, symbols, object_index, *index));
        for local in local_records {
            let canonical = canonical_record_name(objects, symbols, object_index, local);
            let id = if let Some(id) = canonical_records.get(&canonical).copied() {
                id
            } else {
                let id = RecordTypeId::try_from_index(record_order.len())
                    .map_err(|_| LinkError::Overflow("record type IDs"))?;
                canonical_records.insert(canonical, id);
                record_order.push((object_index, local));
                id
            };
            records[object_index][local] = Some(id);
        }

        let mut local_enums = (0..object.enums.len()).collect::<Vec<_>>();
        local_enums
            .sort_by_key(|index| canonical_enum_name(objects, symbols, object_index, *index));
        for local in local_enums {
            let canonical = canonical_enum_name(objects, symbols, object_index, local);
            let id = if let Some(id) = canonical_enums.get(&canonical).copied() {
                id
            } else {
                let id = EnumTypeId::try_from_index(enum_order.len())
                    .map_err(|_| LinkError::Overflow("enum type IDs"))?;
                canonical_enums.insert(canonical, id);
                enum_order.push((object_index, local));
                id
            };
            enums[object_index][local] = Some(id);
        }
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
    let mut canonical_variants = BTreeMap::new();
    let mut next = 0;
    for (object_index, object) in objects.iter().enumerate() {
        for (local, (enumeration, variant_ids)) in object
            .enums
            .iter()
            .zip(&mut variants[object_index])
            .enumerate()
        {
            let canonical = canonical_enum_name(objects, symbols, object_index, local);
            for (variant, target) in variant_ids
                .iter_mut()
                .enumerate()
                .take(enumeration.variants.len())
            {
                let key = (canonical.clone(), variant);
                let id = if let Some(id) = canonical_variants.get(&key).copied() {
                    id
                } else {
                    let id = EnumVariantId::try_from_index(next)
                        .map_err(|_| LinkError::Overflow("enum variant IDs"))?;
                    canonical_variants.insert(key, id);
                    next += 1;
                    id
                };
                *target = Some(id);
            }
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

fn canonical_record_name(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
    object: usize,
    local: usize,
) -> String {
    symbols.canonical_target_name(
        objects,
        object,
        DefinitionTarget::Record(u32::try_from(local).unwrap_or(u32::MAX)),
        &objects[object].records[local].name,
    )
}

fn canonical_enum_name(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
    object: usize,
    local: usize,
) -> String {
    symbols.canonical_target_name(
        objects,
        object,
        DefinitionTarget::Enum(u32::try_from(local).unwrap_or(u32::MAX)),
        &objects[object].enums[local].name,
    )
}
