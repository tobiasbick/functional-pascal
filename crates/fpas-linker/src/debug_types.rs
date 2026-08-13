//! Deterministic relocation of object-local portable debugger types.

use fpas_bytecode::{DebugType, DebugTypeId};
use fpas_unit::object::{DefinitionTarget, ObjectDebugType, RelocatableObject, SymbolKind};

use crate::symbols::{ResolvedTarget, SymbolTable};
use crate::{LinkError, LinkIds};

pub(super) struct DebugTypeIds {
    bases: Vec<u32>,
    lengths: Vec<usize>,
}

impl DebugTypeIds {
    pub(super) fn translate(&self, object: usize, local: u32) -> Result<DebugTypeId, LinkError> {
        if local as usize >= self.lengths[object] {
            return Err(LinkError::Overflow("debug type reference"));
        }
        self.bases[object]
            .checked_add(local)
            .map(DebugTypeId::new)
            .ok_or(LinkError::Overflow("debug type IDs"))
    }
}

pub(super) fn merge(
    objects: &[&RelocatableObject],
    ids: &LinkIds,
    symbols: &SymbolTable,
) -> Result<(DebugTypeIds, Vec<DebugType>), LinkError> {
    let mut bases = Vec::with_capacity(objects.len());
    let mut next = 0_u32;
    for object in objects {
        bases.push(next);
        next = next
            .checked_add(
                u32::try_from(object.debug_types.len())
                    .map_err(|_| LinkError::Overflow("debug type IDs"))?,
            )
            .ok_or(LinkError::Overflow("debug type IDs"))?;
    }
    let maps = DebugTypeIds {
        bases,
        lengths: objects
            .iter()
            .map(|object| object.debug_types.len())
            .collect(),
    };
    let reachable = objects
        .iter()
        .map(|object| reachable_types(object))
        .collect::<Vec<_>>();
    let mut linked = Vec::with_capacity(next as usize);
    for (object_index, object) in objects.iter().enumerate() {
        for (type_index, ty) in object.debug_types.iter().enumerate() {
            match relocate_type(objects, object_index, ty, ids, symbols, &maps) {
                Ok(ty) => linked.push(ty),
                Err(LinkError::MissingDebugLayout { .. })
                    if !reachable[object_index][type_index] =>
                {
                    linked.push(DebugType::Dynamic);
                }
                Err(error) => return Err(error),
            }
        }
    }
    Ok((maps, linked))
}

fn reachable_types(object: &RelocatableObject) -> Vec<bool> {
    let mut reachable = vec![false; object.debug_types.len()];
    let mut pending = object
        .globals
        .iter()
        .map(|global| global.ty)
        .chain(
            object
                .functions
                .iter()
                .flat_map(|function| &function.debug.bindings)
                .map(|binding| binding.ty),
        )
        .chain(
            object
                .functions
                .iter()
                .filter_map(|function| function.debug.result_type),
        )
        .chain(
            object
                .records
                .iter()
                .flat_map(|record| record.field_types.iter().copied()),
        )
        .chain(
            object
                .enums
                .iter()
                .flat_map(|enumeration| &enumeration.variants)
                .flat_map(|variant| variant.field_types.iter().copied()),
        )
        .collect::<Vec<_>>();
    while let Some(id) = pending.pop() {
        let Some(marked) = reachable.get_mut(id as usize) else {
            continue;
        };
        if *marked {
            continue;
        }
        *marked = true;
        let Some(ty) = object.debug_types.get(id as usize) else {
            continue;
        };
        match ty {
            ObjectDebugType::Array(inner)
            | ObjectDebugType::Option(inner)
            | ObjectDebugType::Cell(inner)
            | ObjectDebugType::Task(inner) => pending.push(*inner),
            ObjectDebugType::Dictionary { key, value }
            | ObjectDebugType::Result {
                ok: key,
                error: value,
            } => {
                pending.push(*key);
                pending.push(*value);
            }
            ObjectDebugType::Function { parameters, result } => {
                pending.extend(parameters.iter().copied());
                pending.push(*result);
            }
            ObjectDebugType::Unit
            | ObjectDebugType::Boolean
            | ObjectDebugType::Integer
            | ObjectDebugType::Real
            | ObjectDebugType::String
            | ObjectDebugType::Dynamic
            | ObjectDebugType::Record(_)
            | ObjectDebugType::Enum(_) => {}
        }
    }
    reachable
}

fn relocate_type(
    objects: &[&RelocatableObject],
    object: usize,
    ty: &ObjectDebugType,
    ids: &LinkIds,
    symbols: &SymbolTable,
    maps: &DebugTypeIds,
) -> Result<DebugType, LinkError> {
    let child = |local| maps.translate(object, local);
    Ok(match ty {
        ObjectDebugType::Unit => DebugType::Unit,
        ObjectDebugType::Boolean => DebugType::Boolean,
        ObjectDebugType::Integer => DebugType::Integer,
        ObjectDebugType::Real => DebugType::Real,
        ObjectDebugType::String => DebugType::String,
        ObjectDebugType::Dynamic => DebugType::Dynamic,
        ObjectDebugType::Array(inner) => DebugType::Array(child(*inner)?),
        ObjectDebugType::Dictionary { key, value } => DebugType::Dictionary {
            key: child(*key)?,
            value: child(*value)?,
        },
        ObjectDebugType::Result { ok, error } => DebugType::Result {
            ok: child(*ok)?,
            error: child(*error)?,
        },
        ObjectDebugType::Option(inner) => DebugType::Option(child(*inner)?),
        ObjectDebugType::Function { parameters, result } => DebugType::Function {
            parameters: parameters
                .iter()
                .map(|parameter| child(*parameter))
                .collect::<Result<Vec<_>, _>>()?,
            result: child(*result)?,
        },
        ObjectDebugType::Record(name) => {
            let resolved = resolve_record(objects, object, name, symbols)?;
            let DefinitionTarget::Record(local) = resolved.target else {
                return Err(LinkError::Overflow("debug record type"));
            };
            DebugType::Record(
                ids.layouts.records[resolved.object][local as usize]
                    .ok_or(LinkError::Overflow("debug record type ID"))?,
            )
        }
        ObjectDebugType::Enum(name) => {
            let resolved = resolve_enum(objects, object, name, symbols)?;
            let DefinitionTarget::Enum(local) = resolved.target else {
                return Err(LinkError::Overflow("debug enum type"));
            };
            DebugType::Enum(
                ids.layouts.enums[resolved.object][local as usize]
                    .ok_or(LinkError::Overflow("debug enum type ID"))?,
            )
        }
        ObjectDebugType::Cell(inner) => DebugType::Cell(child(*inner)?),
        ObjectDebugType::Task(inner) => DebugType::Task(child(*inner)?),
    })
}

fn resolve_record(
    objects: &[&RelocatableObject],
    object: usize,
    name: &str,
    symbols: &SymbolTable,
) -> Result<ResolvedTarget, LinkError> {
    if let Some(local) = objects[object]
        .records
        .iter()
        .position(|record| layout_name_matches(&record.name, name))
    {
        return Ok(ResolvedTarget {
            object,
            target: DefinitionTarget::Record(
                u32::try_from(local).map_err(|_| LinkError::Overflow("debug record type"))?,
            ),
        });
    }
    symbols
        .resolve_name(objects, name, SymbolKind::Record)
        .ok_or_else(|| LinkError::MissingDebugLayout {
            owner: objects[object].owner.clone(),
            name: name.to_string(),
            kind: SymbolKind::Record,
        })
}

fn resolve_enum(
    objects: &[&RelocatableObject],
    object: usize,
    name: &str,
    symbols: &SymbolTable,
) -> Result<ResolvedTarget, LinkError> {
    if let Some(local) = objects[object]
        .enums
        .iter()
        .position(|enumeration| layout_name_matches(&enumeration.name, name))
    {
        return Ok(ResolvedTarget {
            object,
            target: DefinitionTarget::Enum(
                u32::try_from(local).map_err(|_| LinkError::Overflow("debug enum type"))?,
            ),
        });
    }
    symbols
        .resolve_name(objects, name, SymbolKind::Enum)
        .ok_or_else(|| LinkError::MissingDebugLayout {
            owner: objects[object].owner.clone(),
            name: name.to_string(),
            kind: SymbolKind::Enum,
        })
}

fn layout_name_matches(candidate: &str, expected: &str) -> bool {
    candidate.eq_ignore_ascii_case(expected)
        || candidate
            .rsplit('.')
            .next()
            .is_some_and(|short| short.eq_ignore_ascii_case(expected))
}
