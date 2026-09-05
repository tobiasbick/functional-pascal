//! Validation for portable debugger type graphs and metadata references.

use crate::{DebugType, DebugTypeId};

use super::{ValidationError, ValidationErrorKind};

const MAX_DEBUG_TYPE_DEPTH: usize = 64;

/// Validates portable type nodes, shared graph depth, and metadata references.
#[cold]
pub(super) fn validate_debug_types(executable: &crate::Executable) -> Result<(), ValidationError> {
    let mut depths = vec![Depth::Unknown; executable.debug_types.len()];
    for (index, ty) in executable.debug_types.iter().enumerate() {
        validate_node(executable, ty)?;
        let id = DebugTypeId::try_from_index(index).map_err(|_| {
            ValidationError::executable(ValidationErrorKind::ResourceLimit {
                resource: "debug types",
                actual: executable.debug_types.len(),
                maximum: crate::limits::MAX_DEBUG_TYPES,
            })
        })?;
        validate_depth(executable, id, 0, &mut depths)?;
    }
    for global in &executable.globals {
        validate_type_reference(executable, global.ty, "global type")?;
    }
    for record in &executable.records {
        for field in &record.fields {
            validate_type_reference(executable, field.ty, "record field type")?;
        }
    }
    for variant in &executable.enum_variants {
        if variant.fields.len() != variant.field_types.len() {
            return Err(ValidationError::executable(
                ValidationErrorKind::DebugTypeShape {
                    owner: "enum variant fields",
                    names: variant.fields.len(),
                    types: variant.field_types.len(),
                },
            ));
        }
        for ty in &variant.field_types {
            validate_type_reference(executable, *ty, "enum field type")?;
        }
    }
    for function in &executable.functions {
        for binding in &function.debug.bindings {
            validate_type_reference(executable, binding.ty, "debug binding type")?;
        }
        if let Some(result_type) = function.debug.result_type {
            validate_type_reference(executable, result_type, "function result type")?;
        }
    }
    Ok(())
}

fn validate_node(executable: &crate::Executable, ty: &DebugType) -> Result<(), ValidationError> {
    match ty {
        DebugType::Array(inner)
        | DebugType::Option(inner)
        | DebugType::Cell(inner)
        | DebugType::Task(inner) => validate_type_reference(executable, *inner, "debug type child"),
        DebugType::Dictionary { key, value } => {
            validate_type_reference(executable, *key, "dictionary key type")?;
            validate_type_reference(executable, *value, "dictionary value type")
        }
        DebugType::Result { ok, error } => {
            validate_type_reference(executable, *ok, "result success type")?;
            validate_type_reference(executable, *error, "result error type")
        }
        DebugType::Function { parameters, result } => {
            super::limit(
                "debug function parameters",
                parameters.len(),
                crate::limits::MAX_CALL_ARGUMENTS,
            )?;
            for parameter in parameters {
                validate_type_reference(executable, *parameter, "function parameter type")?;
            }
            validate_type_reference(executable, *result, "function result type")
        }
        DebugType::Record(layout) => validate_layout_reference(
            "record layouts",
            "debug record type",
            u64::from(layout.get()),
            executable.records.len(),
        ),
        DebugType::Enum(layout) => validate_layout_reference(
            "enum layouts",
            "debug enum type",
            u64::from(layout.get()),
            executable.enums.len(),
        ),
        DebugType::Unit
        | DebugType::Boolean
        | DebugType::Integer
        | DebugType::Real
        | DebugType::String
        | DebugType::Dynamic => Ok(()),
    }
}

#[derive(Clone, Copy)]
enum Depth {
    Unknown,
    Visiting,
    Known(u8),
}

fn validate_depth(
    executable: &crate::Executable,
    id: DebugTypeId,
    depth: usize,
    depths: &mut [Depth],
) -> Result<u8, ValidationError> {
    let too_deep = || {
        ValidationError::executable(ValidationErrorKind::DebugTypeDepth {
            actual: MAX_DEBUG_TYPE_DEPTH + 1,
            maximum: MAX_DEBUG_TYPE_DEPTH,
        })
    };
    if depth > MAX_DEBUG_TYPE_DEPTH {
        return Err(too_deep());
    }
    let Some(ty) = executable.debug_types.get(id.get() as usize) else {
        return validate_type_reference(executable, id, "debug type").map(|()| 0);
    };
    match depths[id.get() as usize] {
        Depth::Visiting => {
            return Err(ValidationError::executable(
                ValidationErrorKind::DebugTypeCycle { actual: id.get() },
            ));
        }
        Depth::Known(height) => {
            // A shared suffix can be reached later through a longer prefix.
            return if depth + usize::from(height) <= MAX_DEBUG_TYPE_DEPTH {
                Ok(height)
            } else {
                Err(too_deep())
            };
        }
        Depth::Unknown => {}
    }
    depths[id.get() as usize] = Depth::Visiting;
    let mut height = 0;
    for child in direct_children(ty) {
        let child_height = validate_depth(executable, child, depth + 1, depths)?;
        height = height.max(child_height + 1);
    }
    depths[id.get() as usize] = Depth::Known(height);
    Ok(height)
}

fn direct_children(ty: &DebugType) -> Vec<DebugTypeId> {
    match ty {
        DebugType::Array(inner)
        | DebugType::Option(inner)
        | DebugType::Cell(inner)
        | DebugType::Task(inner) => vec![*inner],
        DebugType::Dictionary { key, value } => vec![*key, *value],
        DebugType::Result { ok, error } => vec![*ok, *error],
        DebugType::Function { parameters, result } => parameters
            .iter()
            .copied()
            .chain(std::iter::once(*result))
            .collect(),
        DebugType::Unit
        | DebugType::Boolean
        | DebugType::Integer
        | DebugType::Real
        | DebugType::String
        | DebugType::Dynamic
        | DebugType::Record(_)
        | DebugType::Enum(_) => Vec::new(),
    }
}

fn validate_type_reference(
    executable: &crate::Executable,
    id: DebugTypeId,
    operand: &'static str,
) -> Result<(), ValidationError> {
    validate_layout_reference(
        "debug types",
        operand,
        u64::from(id.get()),
        executable.debug_types.len(),
    )
}

fn validate_layout_reference(
    table: &'static str,
    operand: &'static str,
    actual: u64,
    length: usize,
) -> Result<(), ValidationError> {
    if usize::try_from(actual).ok().is_some_and(|id| id < length) {
        Ok(())
    } else {
        Err(ValidationError::executable(
            ValidationErrorKind::TableReference {
                table,
                operand,
                actual,
                length,
            },
        ))
    }
}
