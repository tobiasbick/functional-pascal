//! Removal and remapping of layouts unused by a relocatable object.

use std::collections::BTreeSet;

use fpas_unit::object::{DefinitionTarget, RelocatableObject, RelocationKind, SymbolReference};

use super::retained_map;

/// Removes record and enum layouts that the relocatable object cannot reach.
pub(in crate::object) fn prune_unreferenced_layouts(
    object: &mut RelocatableObject,
    retained_names: &BTreeSet<String>,
) -> Result<(), fpas_unit::object::ObjectError> {
    let mut retained_names = retained_names.clone();
    retained_names.extend(object.debug_types.iter().filter_map(|ty| match ty {
        fpas_unit::object::ObjectDebugType::Record(name)
        | fpas_unit::object::ObjectDebugType::Enum(name) => Some(name.to_ascii_lowercase()),
        _ => None,
    }));
    let referenced_records = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Record(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let record_witness = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::RecordField(field) => Some(usize::from(field) + 1),
            _ => None,
        })
        .max()
        .and_then(|required| smallest_record_witness(&object.records, required));
    let referenced_enums = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::EnumVariant {
                enumeration: SymbolReference::Local(index),
                ..
            } => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let enum_witness = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::EnumField(field) => Some(usize::from(field) + 1),
            _ => None,
        })
        .max()
        .and_then(|required| smallest_enum_witness(&object.enums, required));
    let record_witnesses = record_witness.into_iter().collect::<BTreeSet<_>>();
    let enum_witnesses = enum_witness.into_iter().collect::<BTreeSet<_>>();
    let removed_records = removed_layouts(
        &object.records,
        &referenced_records,
        &record_witnesses,
        &retained_names,
    )?;
    let removed_enums = removed_layouts(
        &object.enums,
        &referenced_enums,
        &enum_witnesses,
        &retained_names,
    )?;
    let record_map = retained_map(object.records.len(), removed_records.iter().copied())?;
    let enum_map = retained_map(object.enums.len(), removed_enums.iter().copied())?;

    object.records = object
        .records
        .iter()
        .enumerate()
        .filter(|(index, _)| !removed_records.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, layout)| layout.clone())
        .collect();
    object.enums = object
        .enums
        .iter()
        .enumerate()
        .filter(|(index, _)| !removed_enums.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, layout)| layout.clone())
        .collect();
    object
        .definitions
        .retain_mut(|definition| match definition.target {
            DefinitionTarget::Record(index) => {
                let defines_layout = referenced_records.contains(&index)
                    || retained_names.contains(
                        &definition
                            .name
                            .rsplit('.')
                            .next()
                            .unwrap_or(&definition.name)
                            .to_ascii_lowercase(),
                    )
                    || retained_names.contains(&definition.name.to_ascii_lowercase());
                defines_layout
                    && record_map[index as usize]
                        .map(|mapped| definition.target = DefinitionTarget::Record(mapped))
                        .is_some()
            }
            DefinitionTarget::Enum(index) => {
                let defines_layout = referenced_enums.contains(&index)
                    || retained_names.contains(
                        &definition
                            .name
                            .rsplit('.')
                            .next()
                            .unwrap_or(&definition.name)
                            .to_ascii_lowercase(),
                    )
                    || retained_names.contains(&definition.name.to_ascii_lowercase());
                defines_layout
                    && enum_map[index as usize]
                        .map(|mapped| definition.target = DefinitionTarget::Enum(mapped))
                        .is_some()
            }
            DefinitionTarget::Function(_) | DefinitionTarget::Global(_) => true,
        });
    for relocation in &mut object.relocations {
        match &mut relocation.kind {
            RelocationKind::Record(SymbolReference::Local(index)) => {
                *index = record_map[*index as usize].ok_or(
                    fpas_unit::object::ObjectError::InvalidTableReference(
                        "pruned referenced record layout",
                    ),
                )?;
            }
            RelocationKind::EnumVariant {
                enumeration: SymbolReference::Local(index),
                ..
            } => {
                *index = enum_map[*index as usize].ok_or(
                    fpas_unit::object::ObjectError::InvalidTableReference(
                        "pruned referenced enum layout",
                    ),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn removed_layouts<T>(
    layouts: &[T],
    referenced: &BTreeSet<u32>,
    witnesses: &BTreeSet<u32>,
    retained_names: &BTreeSet<String>,
) -> Result<BTreeSet<u32>, fpas_unit::object::ObjectError>
where
    T: LayoutName,
{
    layouts
        .iter()
        .enumerate()
        .filter_map(|(index, layout)| {
            let index = u32::try_from(index).ok()?;
            let name = layout.layout_name().to_ascii_lowercase();
            let short_name = name.rsplit('.').next().unwrap_or(&name);
            (!referenced.contains(&index)
                && !witnesses.contains(&index)
                && !retained_names.contains(&name)
                && !retained_names.contains(short_name))
            .then_some(Ok(index))
        })
        .collect()
}

fn smallest_record_witness(
    layouts: &[fpas_unit::object::ObjectRecordLayout],
    required: usize,
) -> Option<u32> {
    layouts
        .iter()
        .enumerate()
        .filter(|(_, layout)| layout.fields.len() >= required)
        .min_by_key(|(_, layout)| (layout.fields.len(), layout.name.to_ascii_lowercase()))
        .and_then(|(index, _)| u32::try_from(index).ok())
}

fn smallest_enum_witness(
    layouts: &[fpas_unit::object::ObjectEnumLayout],
    required: usize,
) -> Option<u32> {
    layouts
        .iter()
        .enumerate()
        .filter_map(|(index, layout)| {
            let capacity = layout
                .variants
                .iter()
                .map(|variant| variant.fields.len())
                .max()
                .unwrap_or(0);
            (capacity >= required).then_some((index, capacity, layout.name.to_ascii_lowercase()))
        })
        .min_by(|left, right| (left.1, &left.2).cmp(&(right.1, &right.2)))
        .and_then(|(index, _, _)| u32::try_from(index).ok())
}

trait LayoutName {
    fn layout_name(&self) -> &str;
}

impl LayoutName for fpas_unit::object::ObjectRecordLayout {
    fn layout_name(&self) -> &str {
        &self.name
    }
}

impl LayoutName for fpas_unit::object::ObjectEnumLayout {
    fn layout_name(&self) -> &str {
        &self.name
    }
}
