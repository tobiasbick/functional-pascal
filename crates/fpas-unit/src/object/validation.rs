//! Canonical object-table, source-run, and relocation coverage validation.

use std::collections::BTreeSet;

use fpas_bytecode::{Instruction, Opcode};

use crate::object::{
    ImportShape, ObjectDebugType, ObjectError, ObjectFunction, ObjectGlobal, ObjectRecordLayout,
    RelocationKind,
};

#[derive(Clone, Copy)]
pub(super) enum RelocationCategory {
    Constant,
    Function,
    Global,
    Record,
    RecordField,
    EnumVariant,
    EnumField,
    CodeAddress,
}

impl RelocationCategory {
    pub(super) fn matches(self, kind: &RelocationKind) -> bool {
        matches!(
            (self, kind),
            (Self::Constant, RelocationKind::Constant(_))
                | (Self::Function, RelocationKind::Function(_))
                | (Self::Global, RelocationKind::Global(_))
                | (Self::Record, RelocationKind::Record(_))
                | (Self::RecordField, RelocationKind::RecordField(_))
                | (Self::EnumVariant, RelocationKind::EnumVariant { .. })
                | (Self::EnumField, RelocationKind::EnumField(_))
                | (Self::CodeAddress, RelocationKind::CodeAddress(_))
        )
    }
}

/// Returns the relocation category required by one object instruction.
pub(super) fn relocation_category(
    instruction: Instruction,
) -> Result<Option<RelocationCategory>, ObjectError> {
    let opcode = instruction
        .opcode()
        .map_err(|error| ObjectError::Instruction(error.to_string()))?;
    Ok(match opcode {
        Opcode::LoadConstant => Some(RelocationCategory::Constant),
        Opcode::Jump | Opcode::BranchIfFalse | Opcode::BranchIfTrue => {
            Some(RelocationCategory::CodeAddress)
        }
        Opcode::CallDirect | Opcode::MakeClosure => Some(RelocationCategory::Function),
        Opcode::LoadGlobal | Opcode::StoreGlobal | Opcode::StoreGlobalIndexPath => {
            Some(RelocationCategory::Global)
        }
        Opcode::MakeRecord => Some(RelocationCategory::Record),
        Opcode::LoadField | Opcode::StoreField => Some(RelocationCategory::RecordField),
        Opcode::MakeEnum | Opcode::TestVariant => Some(RelocationCategory::EnumVariant),
        Opcode::LoadEnumField => Some(RelocationCategory::EnumField),
        _ => None,
    })
}

pub(super) fn validate_source_runs(
    function: &ObjectFunction,
    source_count: usize,
) -> Result<(), ObjectError> {
    let mut previous = None;
    for run in &function.source_runs {
        if previous.is_some_and(|start| start >= run.instruction_start)
            || run.instruction_start as usize >= function.code.len()
            || run.source as usize >= source_count
            || run.line == 0
            || run.column == 0
        {
            return Err(ObjectError::InvalidSourceRun {
                function: function.name.clone(),
                instruction: run.instruction_start,
            });
        }
        previous = Some(run.instruction_start);
    }
    Ok(())
}

pub(super) fn validate_debug_info(
    function: &ObjectFunction,
    source_count: usize,
    debug_type_count: usize,
) -> Result<(), ObjectError> {
    for (index, scope) in function.debug.scopes.iter().enumerate() {
        let valid = usize::try_from(scope.id).ok() == Some(index)
            && match (scope.id, scope.parent) {
                (0, None) => true,
                (_, Some(parent)) => parent < scope.id,
                _ => false,
            };
        if !valid {
            return Err(ObjectError::InvalidTableReference("debug scope"));
        }
    }
    let valid_scope = |scope: u32| {
        usize::try_from(scope)
            .ok()
            .is_some_and(|scope| scope < function.debug.scopes.len())
    };
    for binding in &function.debug.bindings {
        if binding.name.is_empty()
            || binding.type_name.is_empty()
            || binding.ty as usize >= debug_type_count
            || binding.register >= function.register_count
            || !valid_scope(binding.scope)
        {
            return Err(ObjectError::InvalidTableReference("debug binding"));
        }
        if let Some(location) = binding.declaration {
            validate_debug_location(location, source_count)?;
        }
    }
    if let Some(ty) = function.debug.result_type
        && (ty as usize) >= debug_type_count
    {
        return Err(ObjectError::InvalidTableReference("function result type"));
    }
    let mut previous = None;
    for point in &function.debug.sequence_points {
        if previous.is_some_and(|address| address >= point.instruction_start)
            || point.instruction_start as usize >= function.code.len()
            || !valid_scope(point.scope)
        {
            return Err(ObjectError::InvalidSourceRun {
                function: function.name.clone(),
                instruction: point.instruction_start,
            });
        }
        validate_debug_location(point.location, source_count)?;
        previous = Some(point.instruction_start);
    }
    Ok(())
}

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
    for root in 0..types.len() {
        validate_debug_type_depth(types, root, &mut Vec::new(), 0)?;
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

fn validate_debug_type_depth(
    types: &[ObjectDebugType],
    current: usize,
    path: &mut Vec<usize>,
    depth: usize,
) -> Result<(), ObjectError> {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH || path.contains(&current) {
        return Err(ObjectError::InvalidTableReference("debug type graph"));
    }
    path.push(current);
    for child in debug_type_children(&types[current]) {
        validate_debug_type_depth(types, child as usize, path, depth + 1)?;
    }
    path.pop();
    Ok(())
}

fn debug_type_children(ty: &ObjectDebugType) -> Vec<u32> {
    match ty {
        ObjectDebugType::Array(inner)
        | ObjectDebugType::Option(inner)
        | ObjectDebugType::Cell(inner)
        | ObjectDebugType::Task(inner) => vec![*inner],
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

fn validate_debug_location(
    location: crate::object::ObjectDebugLocation,
    source_count: usize,
) -> Result<(), ObjectError> {
    if location.source as usize >= source_count || location.line == 0 || location.column == 0 {
        return Err(ObjectError::InvalidTableReference("debug source location"));
    }
    Ok(())
}

pub(super) fn validate_import_shape(shape: &ImportShape) -> Result<(), ObjectError> {
    match shape {
        ImportShape::Record { fields } => validate_unique_names(fields.iter()),
        ImportShape::Enum { variants } => {
            validate_unique_names(variants.iter().map(|(name, _)| name))?;
            for (_, fields) in variants {
                validate_unique_names(fields.iter())?;
            }
            Ok(())
        }
        ImportShape::Function { .. } | ImportShape::Global { .. } => Ok(()),
    }
}

pub(super) fn validate_unique_names<'a>(
    names: impl Iterator<Item = &'a String>,
) -> Result<(), ObjectError> {
    let mut seen = BTreeSet::new();
    for name in names {
        validate_name(name)?;
        if !seen.insert(name) {
            return Err(ObjectError::DuplicateName(name.clone()));
        }
    }
    Ok(())
}

pub(super) fn validate_name_order<'a>(
    names: impl Iterator<Item = &'a String>,
    table: &'static str,
) -> Result<(), ObjectError> {
    let mut previous: Option<&str> = None;
    for name in names {
        if previous.is_some_and(|value| value >= name.as_str()) {
            return Err(ObjectError::NonDeterministicOrder(table));
        }
        previous = Some(name);
    }
    Ok(())
}

pub(super) fn validate_name(name: &str) -> Result<(), ObjectError> {
    if name.is_empty() || name != super::canonical(name) {
        Err(ObjectError::NonCanonicalName(name.to_string()))
    } else {
        Ok(())
    }
}
