//! Conversion of verified executable operands to object-local relocations.

use fpas_bytecode::{DebugType, Instruction, Opcode};

use crate::object::{
    ObjectDebugType, ObjectEnumLayout, ObjectError, RelocationKind, SymbolReference,
};

/// Returns the object-local relocation required by one verified instruction.
pub(super) fn relocation_for_instruction(
    instruction: Instruction,
    variants: &[fpas_bytecode::EnumVariant],
    enums: &[ObjectEnumLayout],
) -> Result<Option<RelocationKind>, ObjectError> {
    let opcode = instruction
        .opcode()
        .map_err(|error| ObjectError::Instruction(error.to_string()))?;
    if matches!(
        opcode,
        Opcode::LoadConstant
            | Opcode::Jump
            | Opcode::BranchIfFalse
            | Opcode::BranchIfTrue
            | Opcode::LoadGlobal
            | Opcode::StoreGlobal
    ) {
        let operands = instruction
            .abx_operands()
            .map_err(|error| ObjectError::Instruction(error.to_string()))?;
        return Ok(Some(match opcode {
            Opcode::LoadConstant => RelocationKind::Constant(operands.bx),
            Opcode::LoadGlobal | Opcode::StoreGlobal => {
                RelocationKind::Global(SymbolReference::Local(operands.bx))
            }
            _ => RelocationKind::CodeAddress(operands.bx),
        }));
    }
    let operands = instruction
        .abc_operands()
        .map_err(|error| ObjectError::Instruction(error.to_string()))?;
    Ok(match opcode {
        Opcode::StoreGlobalIndexPath => Some(RelocationKind::Global(SymbolReference::Local(
            u32::from(operands.b),
        ))),
        Opcode::CallDirect | Opcode::MakeClosure => Some(RelocationKind::Function(
            SymbolReference::Local(u32::from(operands.b)),
        )),
        Opcode::MakeRecord => Some(RelocationKind::Record(SymbolReference::Local(u32::from(
            operands.b,
        )))),
        Opcode::LoadField => Some(RelocationKind::RecordField(operands.c)),
        Opcode::StoreField => Some(RelocationKind::RecordField(operands.b)),
        Opcode::MakeEnum | Opcode::TestVariant => {
            let index = if opcode == Opcode::MakeEnum {
                operands.b
            } else {
                operands.c
            };
            let variant = variants
                .get(usize::from(index))
                .ok_or(ObjectError::InvalidTableReference("enum variant"))?;
            let layout = enums
                .get(usize::from(variant.owner.get()))
                .ok_or(ObjectError::InvalidTableReference("enum layout"))?;
            let variant_name = layout
                .variants
                .get(
                    variants[..usize::from(index)]
                        .iter()
                        .filter(|item| item.owner == variant.owner)
                        .count(),
                )
                .ok_or(ObjectError::InvalidTableReference("enum variant name"))?
                .name
                .clone();
            Some(RelocationKind::EnumVariant {
                enumeration: SymbolReference::Local(u32::from(variant.owner.get())),
                variant: super::canonical(&variant_name),
            })
        }
        Opcode::LoadEnumField => Some(RelocationKind::EnumField(operands.c)),
        _ => None,
    })
}

pub(super) fn localize_branch(
    instruction: Instruction,
    function_start: u32,
) -> Result<Instruction, ObjectError> {
    let opcode = instruction
        .opcode()
        .map_err(|error| ObjectError::Instruction(error.to_string()))?;
    if !matches!(
        opcode,
        Opcode::Jump | Opcode::BranchIfFalse | Opcode::BranchIfTrue
    ) {
        return Ok(instruction);
    }
    let operands = instruction
        .abx_operands()
        .map_err(|error| ObjectError::Instruction(error.to_string()))?;
    let target =
        operands
            .bx
            .checked_sub(function_start)
            .ok_or(ObjectError::BranchOutsideFunction {
                target: operands.bx,
                function_start,
            })?;
    Instruction::abx(opcode, operands.a, target)
        .map_err(|error| ObjectError::Instruction(error.to_string()))
}

pub(super) fn object_debug_type(
    ty: &DebugType,
    executable: &fpas_bytecode::Executable,
) -> Result<ObjectDebugType, ObjectError> {
    let id = |id: fpas_bytecode::DebugTypeId| id.get();
    let layout_name = |name: fpas_bytecode::StringId| {
        executable
            .strings
            .get(name)
            .map(str::to_owned)
            .ok_or(ObjectError::InvalidTableReference("debug type layout name"))
    };
    Ok(match ty {
        DebugType::Unit => ObjectDebugType::Unit,
        DebugType::Boolean => ObjectDebugType::Boolean,
        DebugType::Integer => ObjectDebugType::Integer,
        DebugType::Real => ObjectDebugType::Real,
        DebugType::String => ObjectDebugType::String,
        DebugType::Dynamic => ObjectDebugType::Dynamic,
        DebugType::Array(inner) => ObjectDebugType::Array(id(*inner)),
        DebugType::Dictionary { key, value } => ObjectDebugType::Dictionary {
            key: id(*key),
            value: id(*value),
        },
        DebugType::Result { ok, error } => ObjectDebugType::Result {
            ok: id(*ok),
            error: id(*error),
        },
        DebugType::Option(inner) => ObjectDebugType::Option(id(*inner)),
        DebugType::Function { parameters, result } => ObjectDebugType::Function {
            parameters: parameters.iter().copied().map(id).collect(),
            result: id(*result),
        },
        DebugType::Record(layout) => {
            let record = executable
                .records
                .get(usize::from(layout.get()))
                .ok_or(ObjectError::InvalidTableReference("debug record layout"))?;
            ObjectDebugType::Record(super::canonical(&layout_name(record.name)?))
        }
        DebugType::Enum(layout) => {
            let enumeration = executable
                .enums
                .get(usize::from(layout.get()))
                .ok_or(ObjectError::InvalidTableReference("debug enum layout"))?;
            ObjectDebugType::Enum(super::canonical(&layout_name(enumeration.name)?))
        }
        DebugType::Cell(inner) => ObjectDebugType::Cell(id(*inner)),
        DebugType::Task(inner) => ObjectDebugType::Task(id(*inner)),
        DebugType::Channel(inner) => ObjectDebugType::Channel(id(*inner)),
    })
}
