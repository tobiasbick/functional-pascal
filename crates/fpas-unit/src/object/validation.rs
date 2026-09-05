//! Canonical object-table, source-run, and relocation coverage validation.

use std::collections::BTreeSet;

use fpas_bytecode::{Instruction, Opcode};

use crate::object::{ImportShape, ObjectError, ObjectFunction, RelocationKind};

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
    functions: &[ObjectFunction],
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
        if let Some(initializer) = binding.initializer_start {
            let instruction = function
                .code
                .get(initializer as usize)
                .copied()
                .map(fpas_bytecode::Instruction::from_word)
                .ok_or(ObjectError::InvalidTableReference(
                    "debug binding initializer instruction",
                ))?;
            if instruction.opcode().ok() != Some(fpas_bytecode::Opcode::Move)
                || instruction.abc_payload().a != binding.register
            {
                return Err(ObjectError::InvalidTableReference(
                    "debug binding initializer store",
                ));
            }
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
    validate_capture_provenance(function, functions, debug_type_count)?;
    Ok(())
}

fn validate_capture_provenance(
    function: &ObjectFunction,
    functions: &[ObjectFunction],
    debug_type_count: usize,
) -> Result<(), ObjectError> {
    let sources = &function.debug.capture_sources;
    if function.capture_count == 0 {
        if function.debug.lexical_owner.is_some() || !sources.is_empty() {
            return Err(ObjectError::InvalidTableReference("capture provenance"));
        }
        return Ok(());
    }
    let Some(owner_index) = function.debug.lexical_owner else {
        return Err(ObjectError::InvalidTableReference("lexical owner"));
    };
    let Some(owner) = functions.get(owner_index as usize) else {
        return Err(ObjectError::InvalidTableReference("lexical owner"));
    };
    if sources.len() != usize::from(function.capture_count) {
        return Err(ObjectError::InvalidTableReference("capture source count"));
    }
    for source in sources {
        if source.binding as usize >= owner.debug.bindings.len()
            || source.ty as usize >= debug_type_count
        {
            return Err(ObjectError::InvalidTableReference("capture source"));
        }
        let binding = &owner.debug.bindings[source.binding as usize];
        if binding.ty != source.ty {
            return Err(ObjectError::InvalidTableReference("capture source type"));
        }
        match source.kind {
            crate::object::ObjectCaptureKind::Value if binding.cell_backed || binding.hidden => {
                return Err(ObjectError::InvalidTableReference("value capture source"));
            }
            crate::object::ObjectCaptureKind::Cell
            | crate::object::ObjectCaptureKind::EnclosingCell
                if !binding.cell_backed =>
            {
                return Err(ObjectError::InvalidTableReference("cell capture source"));
            }
            _ => {}
        }
    }
    Ok(())
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
