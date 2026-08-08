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
