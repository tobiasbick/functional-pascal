//! Relocatable-object appending and address rebasing.

use std::collections::{HashMap, HashSet};

use fpas_bytecode::{Chunk, SourceLocation};
use fpas_unit::object::{ObjectLocation, RelocatableObject, Relocation};

use crate::LinkError;
use crate::operands::relocate_instruction;

pub(super) fn validate_retained_function_entries(
    object: &RelocatableObject,
) -> Result<(), LinkError> {
    let retained_code = object.code.len() - 1;
    for (name, function) in &object.functions {
        if function.code_start as usize >= retained_code {
            return Err(LinkError::StrippedFunctionEntry {
                owner: object.owner.clone(),
                name: name.clone(),
                offset: function.code_start,
                retained_code,
            });
        }
    }
    Ok(())
}

pub(super) fn append_object(
    chunk: &mut Chunk,
    object: &RelocatableObject,
    retain_halt: bool,
) -> Result<(), LinkError> {
    let code_base = u32::try_from(chunk.len()).map_err(|_| LinkError::Overflow("code"))?;
    let mut constant_map = Vec::with_capacity(object.constants.len());
    for constant in &object.constants {
        let index = chunk
            .add_constant(constant.to_value())
            .map_err(|error| LinkError::ConstantPool(error.to_string()))?;
        constant_map.push(index);
    }

    let relocation_by_instruction = relocation_map(object);
    let code_length = object.code.len() - usize::from(!retain_halt);
    for (offset, (op, location)) in object
        .code
        .iter()
        .zip(&object.locations)
        .take(code_length)
        .enumerate()
    {
        let mut relocated = *op;
        if let Some(relocations) = relocation_by_instruction.get(&(offset as u32)) {
            for relocation in relocations {
                relocate_instruction(&mut relocated, relocation.kind, &constant_map, code_base)
                    .map_err(|()| LinkError::InvalidRelocation {
                        owner: object.owner.clone(),
                        instruction: offset as u32,
                    })?;
            }
        }
        chunk.emit(relocated, source_location(*location));
    }

    let mut function_names = HashSet::new();
    for (name, function) in &object.functions {
        let key = name.to_ascii_lowercase();
        if !function_names.insert(key.clone()) || chunk.functions().contains_key(&key) {
            return Err(LinkError::DuplicateFunction(name.clone()));
        }
        let start = code_base
            .checked_add(function.code_start)
            .ok_or(LinkError::Overflow("function address"))?;
        chunk.insert_function(key, start as usize, function.arity);
    }
    Ok(())
}

fn relocation_map(object: &RelocatableObject) -> HashMap<u32, Vec<Relocation>> {
    let mut result = HashMap::<u32, Vec<_>>::new();
    for relocation in &object.relocations {
        result
            .entry(relocation.instruction)
            .or_default()
            .push(*relocation);
    }
    result
}

fn source_location(location: ObjectLocation) -> SourceLocation {
    SourceLocation::new_with_source(location.line, location.column, location.source_id)
}
