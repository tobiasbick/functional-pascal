//! Checked packed-instruction relocation into final numeric tables.

use fpas_bytecode::{Instruction, Opcode};
use fpas_unit::object::{
    DefinitionTarget, RelocatableObject, Relocation, RelocationKind, SymbolKind,
};

use crate::constants::ConstantIds;
use crate::symbols::SymbolTable;
use crate::{LinkError, LinkIds};

#[expect(
    clippy::too_many_arguments,
    reason = "relocation needs all final table mappings"
)]
pub(super) fn relocate(
    objects: &[&RelocatableObject],
    object_index: usize,
    function_index: usize,
    instruction: Instruction,
    relocation: &Relocation,
    code_base: u32,
    symbols: &SymbolTable,
    ids: &LinkIds,
    constants: &ConstantIds,
) -> Result<Instruction, LinkError> {
    let opcode = instruction
        .opcode()
        .map_err(|error| LinkError::Instruction(error.to_string()))?;
    match &relocation.kind {
        RelocationKind::Constant(local) => {
            let mapped = constants
                .maps
                .get(object_index)
                .and_then(|map| map.get(*local as usize))
                .ok_or(LinkError::Overflow("constant reference"))?;
            replace_abx(instruction, opcode, mapped.get())
        }
        RelocationKind::Global(reference) => {
            let resolved = symbols.resolve(object_index, *reference, SymbolKind::Global)?;
            let DefinitionTarget::Global(local) = resolved.target else {
                return Err(LinkError::Overflow("global target"));
            };
            let mapped = ids
                .globals
                .maps
                .get(resolved.object)
                .and_then(|map| map.get(local as usize))
                .and_then(|id| *id)
                .ok_or(LinkError::Overflow("global reference"))?;
            replace_abx(instruction, opcode, mapped.get())
        }
        RelocationKind::CodeAddress(local) => {
            let mapped = code_base
                .checked_add(*local)
                .ok_or(LinkError::Overflow("branch address"))?;
            replace_abx(instruction, opcode, mapped)
        }
        RelocationKind::Function(reference) => {
            let resolved = symbols.resolve(object_index, *reference, SymbolKind::Function)?;
            let DefinitionTarget::Function(local) = resolved.target else {
                return Err(LinkError::Overflow("function target"));
            };
            let mapped = ids
                .functions
                .maps
                .get(resolved.object)
                .and_then(|map| map.get(local as usize))
                .and_then(|id| *id)
                .ok_or(LinkError::Overflow("function reference"))?;
            replace_abc(instruction, opcode, mapped.get())
        }
        RelocationKind::Record(reference) => {
            let resolved = symbols.resolve(object_index, *reference, SymbolKind::Record)?;
            let DefinitionTarget::Record(local) = resolved.target else {
                return Err(LinkError::Overflow("record target"));
            };
            let mapped = ids
                .layouts
                .records
                .get(resolved.object)
                .and_then(|map| map.get(local as usize))
                .and_then(|id| *id)
                .ok_or(LinkError::Overflow("record reference"))?;
            replace_abc(instruction, opcode, mapped.get())
        }
        RelocationKind::RecordField(field) => {
            let maximum = objects
                .iter()
                .flat_map(|object| &object.records)
                .map(|record| record.fields.len())
                .max()
                .unwrap_or(0);
            if usize::from(*field) >= maximum {
                return Err(LinkError::InvalidField {
                    owner: objects[object_index].owner.clone(),
                    field: *field,
                    available: maximum,
                });
            }
            replace_abc(instruction, opcode, *field)
        }
        RelocationKind::EnumVariant {
            enumeration,
            variant,
        } => {
            let resolved = symbols.resolve(object_index, *enumeration, SymbolKind::Enum)?;
            let DefinitionTarget::Enum(local) = resolved.target else {
                return Err(LinkError::Overflow("enum target"));
            };
            let layout = objects[resolved.object]
                .enums
                .get(local as usize)
                .ok_or(LinkError::Overflow("enum layout reference"))?;
            let variant_index = layout
                .variants
                .iter()
                .position(|candidate| candidate.name.eq_ignore_ascii_case(variant))
                .ok_or_else(|| LinkError::MissingVariant {
                    enumeration: layout.name.clone(),
                    variant: variant.clone(),
                })?;
            let mapped = ids.layouts.variants[resolved.object][local as usize][variant_index]
                .ok_or(LinkError::Overflow("enum variant reference"))?;
            replace_abc(instruction, opcode, mapped.get())
        }
        RelocationKind::EnumField(field) => {
            let maximum = objects
                .iter()
                .flat_map(|object| &object.enums)
                .flat_map(|layout| &layout.variants)
                .map(|variant| variant.fields.len())
                .max()
                .unwrap_or(0);
            if usize::from(*field) >= maximum {
                return Err(LinkError::InvalidField {
                    owner: objects[object_index].owner.clone(),
                    field: *field,
                    available: maximum,
                });
            }
            replace_abc(instruction, opcode, *field)
        }
    }
    .map_err(|detail| LinkError::InvalidRelocation {
        owner: objects[object_index].owner.clone(),
        function: u32::try_from(function_index).unwrap_or(u32::MAX),
        instruction: relocation.instruction,
        detail,
    })
}

fn replace_abx(instruction: Instruction, opcode: Opcode, bx: u32) -> Result<Instruction, String> {
    let operands = instruction
        .abx_operands()
        .map_err(|error| error.to_string())?;
    Instruction::abx(opcode, operands.a, bx).map_err(|error| error.to_string())
}

fn replace_abc(
    instruction: Instruction,
    opcode: Opcode,
    mapped: u16,
) -> Result<Instruction, String> {
    let operands = instruction
        .abc_operands()
        .map_err(|error| error.to_string())?;
    let (a, b, c) = match opcode {
        Opcode::CallDirect | Opcode::MakeClosure | Opcode::MakeRecord | Opcode::MakeEnum => {
            (operands.a, mapped, operands.c)
        }
        Opcode::LoadField | Opcode::TestVariant | Opcode::LoadEnumField => {
            (operands.a, operands.b, mapped)
        }
        Opcode::StoreField => (operands.a, mapped, operands.c),
        _ => return Err(format!("opcode {opcode:?} has no relocatable ABC operand")),
    };
    Instruction::abc(opcode, a, b, c, operands.auxiliary).map_err(|error| error.to_string())
}
