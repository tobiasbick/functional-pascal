//! Import-plan lookup and local-table remapping helpers.

mod layout_pruning;

use std::collections::{BTreeMap, BTreeSet};

use fpas_unit::object::{
    DefinitionTarget, ObjectConstant, ObjectImport, RelocatableObject, RelocationKind,
    SymbolReference,
};

use crate::lowering::ImportPlan;

pub(super) use layout_pruning::prune_unreferenced_layouts;

/// Rewrite a relocatable object so planned imports replace local stubs.
pub(super) fn apply_imports(
    object: &mut RelocatableObject,
    plan: ImportPlan,
) -> Result<(), fpas_unit::object::ObjectError> {
    let planned_functions = plan
        .functions
        .iter()
        .map(|(id, _)| id.get())
        .collect::<BTreeSet<_>>();
    let planned_globals = plan
        .globals
        .iter()
        .map(|(id, _)| id.get())
        .collect::<BTreeSet<_>>();
    let planned_records = planned_layouts(
        object.records.iter().map(|record| &record.name),
        &plan.layouts,
        |shape| matches!(shape, fpas_unit::object::ImportShape::Record { .. }),
    )?;
    let planned_enums = planned_layouts(
        object.enums.iter().map(|enumeration| &enumeration.name),
        &plan.layouts,
        |shape| matches!(shape, fpas_unit::object::ImportShape::Enum { .. }),
    )?;
    let referenced_records = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Record(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
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
    let referenced_functions = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Function(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .chain(
            object
                .constants
                .iter()
                .filter_map(|constant| match constant {
                    ObjectConstant::Function {
                        function: SymbolReference::Local(index),
                        ..
                    } => Some(*index),
                    _ => None,
                }),
        )
        .collect::<BTreeSet<_>>();
    let referenced_globals = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Global(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut required_import_names = plan
        .functions
        .iter()
        .filter(|(id, _)| referenced_functions.contains(&id.get()))
        .map(|(_, import)| import.name.clone())
        .chain(
            plan.globals
                .iter()
                .filter(|(id, _)| referenced_globals.contains(&id.get()))
                .map(|(_, import)| import.name.clone()),
        )
        .collect::<BTreeSet<_>>();
    for (index, record) in object.records.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| fpas_unit::object::ObjectError::Overflow("local record index"))?;
        if planned_records.contains(&index) && referenced_records.contains(&index) {
            required_import_names.insert(record.name.clone());
        }
    }
    for (index, enumeration) in object.enums.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| fpas_unit::object::ObjectError::Overflow("local enum index"))?;
        if planned_enums.contains(&index) && referenced_enums.contains(&index) {
            required_import_names.insert(enumeration.name.clone());
        }
    }
    let mut imports = plan
        .functions
        .iter()
        .map(|(_, import)| import)
        .chain(plan.globals.iter().map(|(_, import)| import))
        .chain(plan.layouts.iter())
        .filter(|import| required_import_names.contains(&import.name))
        .cloned()
        .collect::<Vec<_>>();
    imports.sort_by(|left, right| left.name.cmp(&right.name));
    imports.dedup_by(|left, right| left.name == right.name);
    let import_indices = imports
        .iter()
        .enumerate()
        .map(|(index, import)| (import.name.clone(), u32::try_from(index)))
        .map(|(name, index)| {
            index
                .map(|index| (name, index))
                .map_err(|_| fpas_unit::object::ObjectError::Overflow("import index"))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_functions = plan
        .functions
        .iter()
        .filter(|(id, _)| referenced_functions.contains(&id.get()))
        .map(|(id, import)| imported_index(id.get(), import, &import_indices))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_globals = plan
        .globals
        .iter()
        .filter(|(id, _)| referenced_globals.contains(&id.get()))
        .map(|(id, import)| imported_index(id.get(), import, &import_indices))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_records = imported_layouts(
        &planned_records,
        &referenced_records,
        object.records.iter().map(|record| &record.name),
        &import_indices,
    )?;
    let imported_enums = imported_layouts(
        &planned_enums,
        &referenced_enums,
        object.enums.iter().map(|enumeration| &enumeration.name),
        &import_indices,
    )?;
    let function_map = retained_map(object.functions.len(), planned_functions.iter().copied())?;
    let global_map = retained_map(object.globals.len(), planned_globals.iter().copied())?;
    let record_map = retained_map(object.records.len(), planned_records.iter().copied())?;
    let enum_map = retained_map(object.enums.len(), planned_enums.iter().copied())?;

    object.functions = object
        .functions
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !planned_functions.contains(&u32::try_from(*index).unwrap_or(u32::MAX))
        })
        .map(|(_, function)| function.clone())
        .collect();
    for function in &mut object.functions {
        if let Some(owner) = function.debug.lexical_owner {
            function.debug.lexical_owner = usize::try_from(owner)
                .ok()
                .and_then(|index| function_map.get(index).copied().flatten());
        }
    }
    object.globals = object
        .globals
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_globals.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, global)| global.clone())
        .collect();
    object.records = object
        .records
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_records.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, record)| record.clone())
        .collect();
    object.enums = object
        .enums
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_enums.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, enumeration)| enumeration.clone())
        .collect();
    object
        .definitions
        .retain_mut(|definition| match definition.target {
            DefinitionTarget::Function(index) => function_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Function(mapped))
                .is_some(),
            DefinitionTarget::Global(index) => global_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Global(mapped))
                .is_some(),
            DefinitionTarget::Record(index) => record_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Record(mapped))
                .is_some(),
            DefinitionTarget::Enum(index) => enum_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Enum(mapped))
                .is_some(),
        });
    object.relocations.retain_mut(|relocation| {
        let Some(mapped_owner) = function_map[relocation.function as usize] else {
            return false;
        };
        relocation.function = mapped_owner;
        match &mut relocation.kind {
            RelocationKind::Function(SymbolReference::Local(index)) => {
                if let Some(import) = imported_functions.get(index) {
                    relocation.kind = RelocationKind::Function(SymbolReference::Import(*import));
                } else if let Some(mapped) = function_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::Global(SymbolReference::Local(index)) => {
                if let Some(import) = imported_globals.get(index) {
                    relocation.kind = RelocationKind::Global(SymbolReference::Import(*import));
                } else if let Some(mapped) = global_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::Record(SymbolReference::Local(index)) => {
                if let Some(import) = imported_records.get(index) {
                    relocation.kind = RelocationKind::Record(SymbolReference::Import(*import));
                } else if let Some(mapped) = record_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::EnumVariant { enumeration, .. } => {
                if let SymbolReference::Local(index) = enumeration {
                    if let Some(import) = imported_enums.get(index) {
                        *enumeration = SymbolReference::Import(*import);
                    } else if let Some(mapped) = enum_map[*index as usize] {
                        *index = mapped;
                    }
                }
            }
            RelocationKind::Constant(_)
            | RelocationKind::Function(SymbolReference::Import(_))
            | RelocationKind::Global(SymbolReference::Import(_))
            | RelocationKind::Record(SymbolReference::Import(_))
            | RelocationKind::RecordField(_)
            | RelocationKind::EnumField(_)
            | RelocationKind::CodeAddress(_) => {}
        }
        true
    });
    for constant in &mut object.constants {
        if let ObjectConstant::Function {
            function: SymbolReference::Local(index),
            ..
        } = constant
        {
            if let Some(import) = imported_functions.get(index) {
                let task_bound = match constant {
                    ObjectConstant::Function { task_bound, .. } => *task_bound,
                    _ => {
                        return Err(fpas_unit::object::ObjectError::InvalidTableReference(
                            "imported function constant",
                        ));
                    }
                };
                *constant = ObjectConstant::Function {
                    function: SymbolReference::Import(*import),
                    task_bound,
                };
            } else if let Some(mapped) = function_map[*index as usize] {
                *index = mapped;
            }
        }
    }
    object.entry = object.entry.and_then(|index| function_map[index as usize]);
    object.initializer = object
        .initializer
        .and_then(|index| function_map[index as usize]);
    object.imports = imports;
    object
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    object
        .relocations
        .sort_by_key(|relocation| (relocation.function, relocation.instruction));
    Ok(())
}

pub(super) fn planned_layouts<'a>(
    names: impl Iterator<Item = &'a String>,
    planned: &[ObjectImport],
    shape_matches: impl Fn(&fpas_unit::object::ImportShape) -> bool,
) -> Result<BTreeSet<u32>, fpas_unit::object::ObjectError> {
    let planned = planned
        .iter()
        .filter(|import| shape_matches(&import.shape))
        .map(|import| import.name.as_str())
        .collect::<BTreeSet<_>>();
    names
        .enumerate()
        .filter(|(_, name)| planned.contains(name.as_str()))
        .map(|(index, name)| {
            let _ = name;
            u32::try_from(index)
                .map_err(|_| fpas_unit::object::ObjectError::Overflow("local layout index"))
        })
        .collect()
}

pub(super) fn imported_layouts<'a>(
    planned: &BTreeSet<u32>,
    referenced: &BTreeSet<u32>,
    names: impl Iterator<Item = &'a String>,
    import_indices: &BTreeMap<String, u32>,
) -> Result<BTreeMap<u32, u32>, fpas_unit::object::ObjectError> {
    names
        .enumerate()
        .filter_map(|(index, name)| {
            let local = u32::try_from(index).ok()?;
            (planned.contains(&local) && referenced.contains(&local)).then_some((local, name))
        })
        .map(|(local, name)| {
            let import = import_indices.get(name).copied().ok_or(
                fpas_unit::object::ObjectError::InvalidTableReference("planned layout import"),
            )?;
            Ok((local, import))
        })
        .collect()
}

pub(super) fn imported_index(
    id: u32,
    import: &ObjectImport,
    import_indices: &BTreeMap<String, u32>,
) -> Result<(u32, u32), fpas_unit::object::ObjectError> {
    import_indices
        .get(&import.name)
        .copied()
        .map(|index| (id, index))
        .ok_or(fpas_unit::object::ObjectError::InvalidTableReference(
            "planned import",
        ))
}

pub(super) fn retained_map(
    length: usize,
    removed: impl Iterator<Item = u32>,
) -> Result<Vec<Option<u32>>, fpas_unit::object::ObjectError> {
    let removed = removed.collect::<BTreeSet<_>>();
    let mut next = 0_u32;
    (0..length)
        .map(|index| {
            let index = u32::try_from(index)
                .map_err(|_| fpas_unit::object::ObjectError::Overflow("local table index"))?;
            if removed.contains(&index) {
                Ok(None)
            } else {
                let mapped = next;
                next = next
                    .checked_add(1)
                    .ok_or(fpas_unit::object::ObjectError::Overflow(
                        "local table index",
                    ))?;
                Ok(Some(mapped))
            }
        })
        .collect()
}
