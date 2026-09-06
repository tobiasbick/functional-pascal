//! Bounded validation of object debugger types and their shared subgraphs.

use super::validation::validate_name;
use super::{ObjectDebugType, ObjectError, ObjectGlobal, ObjectRecordLayout};

#[cfg(test)]
mod tests;

/// Validates type nodes, graph depth, and metadata type references.
pub(super) fn validate_debug_types(
    types: &[ObjectDebugType],
    globals: &[ObjectGlobal],
    records: &[ObjectRecordLayout],
    enums: &[crate::object::ObjectEnumLayout],
) -> Result<(), ObjectError> {
    if types.len() > fpas_bytecode::limits::MAX_DEBUG_TYPES {
        return Err(ObjectError::InvalidTableReference("debug type count"));
    }
    for ty in types {
        for child in debug_type_children(ty) {
            if child as usize >= types.len() {
                return Err(ObjectError::InvalidTableReference("debug type child"));
            }
        }
        match ty {
            ObjectDebugType::Function { parameters, .. }
                if parameters.len() > fpas_bytecode::limits::MAX_CALL_ARGUMENTS =>
            {
                return Err(ObjectError::InvalidTableReference(
                    "debug function parameter count",
                ));
            }
            ObjectDebugType::Record(name) | ObjectDebugType::Enum(name) => validate_name(name)?,
            _ => {}
        }
    }
    let mut depths = vec![Depth::Unknown; types.len()];
    for root in 0..types.len() {
        validate_debug_type_depth(types, root, &mut depths, 0)?;
    }
    for global in globals {
        validate_debug_type_id(global.ty, types.len())?;
    }
    for record in records {
        if record.fields.len() != record.field_types.len() {
            return Err(ObjectError::InvalidTableReference(
                "record debug field type shape",
            ));
        }
        for ty in &record.field_types {
            validate_debug_type_id(*ty, types.len())?;
        }
    }
    for enumeration in enums {
        for variant in &enumeration.variants {
            if variant.fields.len() != variant.field_types.len() {
                return Err(ObjectError::InvalidTableReference(
                    "enum debug field type shape",
                ));
            }
            for ty in &variant.field_types {
                validate_debug_type_id(*ty, types.len())?;
            }
        }
    }
    Ok(())
}

fn validate_debug_type_id(id: u32, type_count: usize) -> Result<(), ObjectError> {
    if id as usize >= type_count {
        Err(ObjectError::InvalidTableReference("debug type"))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Depth {
    Unknown,
    Visiting,
    Known(u8),
}

fn validate_debug_type_depth(
    types: &[ObjectDebugType],
    current: usize,
    depths: &mut [Depth],
    depth: usize,
) -> Result<u8, ObjectError> {
    const MAX_DEPTH: usize = 64;
    let invalid = || ObjectError::InvalidTableReference("debug type graph");
    if depth > MAX_DEPTH {
        return Err(invalid());
    }
    match depths[current] {
        Depth::Visiting => return Err(invalid()),
        Depth::Known(height) => {
            return if depth + usize::from(height) <= MAX_DEPTH {
                Ok(height)
            } else {
                Err(invalid())
            };
        }
        Depth::Unknown => {}
    }
    depths[current] = Depth::Visiting;
    let mut height = 0;
    for child in debug_type_children(&types[current]) {
        let child_height = validate_debug_type_depth(types, child as usize, depths, depth + 1)?;
        height = height.max(child_height + 1);
    }
    if depth + usize::from(height) > MAX_DEPTH {
        return Err(invalid());
    }
    depths[current] = Depth::Known(height);
    Ok(height)
}

fn debug_type_children(ty: &ObjectDebugType) -> Vec<u32> {
    match ty {
        ObjectDebugType::Array(inner)
        | ObjectDebugType::Option(inner)
        | ObjectDebugType::Cell(inner)
        | ObjectDebugType::Task(inner)
        | ObjectDebugType::Channel(inner) => vec![*inner],
        ObjectDebugType::Dictionary { key, value }
        | ObjectDebugType::Result {
            ok: key,
            error: value,
        } => vec![*key, *value],
        ObjectDebugType::Function { parameters, result } => {
            let mut children = parameters.clone();
            children.push(*result);
            children
        }
        ObjectDebugType::Unit
        | ObjectDebugType::Boolean
        | ObjectDebugType::Integer
        | ObjectDebugType::Real
        | ObjectDebugType::String
        | ObjectDebugType::Dynamic
        | ObjectDebugType::Record(_)
        | ObjectDebugType::Enum(_) => Vec::new(),
    }
}
