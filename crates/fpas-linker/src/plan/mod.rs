//! Canonical linker planning before executable materialization.

mod debug_types;
mod functions;
mod globals;
mod layouts;
mod symbols;

use fpas_bytecode::{DebugType, FunctionId};
use fpas_unit::object::RelocatableObject;

use crate::LinkError;

use self::functions::FunctionIds;
use self::globals::GlobalIds;
use self::layouts::LayoutIds;

pub(crate) use self::debug_types::DebugTypeIds;
pub(crate) use self::symbols::SymbolTable;

/// Object-local identifiers translated into canonical executable identifiers.
pub(super) struct LinkIds {
    pub(super) functions: FunctionIds,
    pub(super) globals: GlobalIds,
    pub(super) layouts: LayoutIds,
}

/// Canonical placement of every linked function in executable code.
pub(super) struct CodeLayout {
    pub(super) starts: Vec<u32>,
    pub(super) bases: Vec<u32>,
    pub(super) length: u32,
    pub(super) initializer_count: u32,
}

/// Validated, canonical linker state shared by all executable materialization concerns.
pub(super) struct LinkPlan<'a> {
    pub(super) objects: Vec<&'a RelocatableObject>,
    pub(super) symbols: SymbolTable,
    pub(super) ids: LinkIds,
    pub(super) debug_type_ids: DebugTypeIds,
    pub(super) linked_debug_types: Vec<DebugType>,
    pub(super) initializer_targets: Vec<FunctionId>,
    pub(super) code_layout: CodeLayout,
}

impl<'a> LinkPlan<'a> {
    /// Validate objects and compute every canonical identifier and code address.
    pub(super) fn build(
        units: &'a [RelocatableObject],
        program: &'a RelocatableObject,
    ) -> Result<Self, LinkError> {
        validate_objects(units, program)?;
        validate_initializers(units)?;

        let objects = units
            .iter()
            .chain(std::iter::once(program))
            .collect::<Vec<_>>();
        let program_index = units.len();
        let entry = program
            .entry
            .and_then(|index| usize::try_from(index).ok())
            .ok_or(LinkError::MissingProgramEntry)?;
        let symbols = SymbolTable::build(&objects)?;
        let ids = LinkIds {
            functions: functions::assign(&objects, program_index, entry, &symbols)?,
            globals: globals::assign(&objects, &symbols)?,
            layouts: layouts::assign(&objects, &symbols)?,
        };
        let (debug_type_ids, linked_debug_types) = debug_types::merge(&objects, &ids, &symbols)?;
        let initializer_targets = initializer_targets(units, &ids)?;
        let code_layout =
            CodeLayout::build(&objects, &ids.functions.order, initializer_targets.len())?;

        Ok(Self {
            objects,
            symbols,
            ids,
            debug_type_ids,
            linked_debug_types,
            initializer_targets,
            code_layout,
        })
    }
}

impl CodeLayout {
    /// Return the first executable instruction address for one canonical function.
    pub(super) fn base_for(&self, function: FunctionId) -> Result<u32, LinkError> {
        self.bases
            .get(function.get() as usize)
            .copied()
            .ok_or(LinkError::Overflow("global initializer function order"))
    }

    fn build(
        objects: &[&RelocatableObject],
        function_order: &[(usize, usize)],
        initializer_count: usize,
    ) -> Result<Self, LinkError> {
        let initializer_count = u32::try_from(initializer_count)
            .map_err(|_| LinkError::Overflow("unit initializer call count"))?;
        let mut starts = Vec::with_capacity(function_order.len());
        let mut bases = Vec::with_capacity(function_order.len());
        let mut length = 0_u32;
        for (final_index, (object, function)) in function_order.iter().enumerate() {
            starts.push(length);
            let prefix = if final_index == 0 {
                initializer_count
            } else {
                0
            };
            bases.push(
                length
                    .checked_add(prefix)
                    .ok_or(LinkError::Overflow("instruction addresses"))?,
            );
            length = length
                .checked_add(prefix)
                .ok_or(LinkError::Overflow("instruction addresses"))?
                .checked_add(
                    u32::try_from(objects[*object].functions[*function].code.len())
                        .map_err(|_| LinkError::Overflow("function code length"))?,
                )
                .ok_or(LinkError::Overflow("instruction addresses"))?;
        }
        Ok(Self {
            starts,
            bases,
            length,
            initializer_count,
        })
    }
}

fn validate_objects(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<(), LinkError> {
    for unit in units {
        unit.validate().map_err(|error| LinkError::InvalidObject {
            owner: unit.owner.clone(),
            detail: error.to_string(),
        })?;
        if unit.entry.is_some() {
            return Err(LinkError::UnitEntry(unit.owner.clone()));
        }
    }
    program
        .validate()
        .map_err(|error| LinkError::InvalidObject {
            owner: program.owner.clone(),
            detail: error.to_string(),
        })
}

fn validate_initializers(units: &[RelocatableObject]) -> Result<(), LinkError> {
    use fpas_unit::object::ObjectReturn;

    for object in units {
        let Some(initializer) = object.initializer else {
            continue;
        };
        let function = &object.functions[initializer as usize];
        let detail = if function.arity != 0 {
            Some("expected zero parameters")
        } else if function.capture_count != 0 {
            Some("expected zero captures")
        } else if function.returns != ObjectReturn::Unit {
            Some("expected Unit return convention")
        } else {
            None
        };
        if let Some(detail) = detail {
            return Err(LinkError::InvalidInitializer {
                owner: object.owner.clone(),
                detail,
            });
        }
    }
    Ok(())
}

fn initializer_targets(
    units: &[RelocatableObject],
    ids: &LinkIds,
) -> Result<Vec<FunctionId>, LinkError> {
    units
        .iter()
        .enumerate()
        .filter_map(|(object_index, object)| {
            object
                .initializer
                .map(|initializer| (object_index, initializer as usize))
        })
        .map(|(object_index, initializer)| {
            ids.functions.maps[object_index][initializer]
                .ok_or(LinkError::Overflow("unit initializer function ID"))
        })
        .collect()
}
