//! Exhaustive relocation discovery for bytecode operands.

use fpas_bytecode::Op;

/// One object-local operand that must be remapped by the final linker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Relocation {
    /// Object-local instruction offset.
    pub instruction: u32,
    /// Operand category and current local value.
    pub kind: RelocationKind,
}

/// Relocatable operand and its position within an opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelocationKind {
    /// Constant-pool operand.
    Constant {
        /// Zero-based constant operand number for multi-constant opcodes.
        operand: u8,
        /// Object-local constant index.
        index: u16,
    },
    /// Absolute object-local instruction target.
    CodeAddress {
        /// Object-local target.
        target: u32,
    },
}

/// Discover every relocatable operand in instruction order.
#[must_use]
pub fn collect_relocations(code: &[Op]) -> Vec<Relocation> {
    let mut relocations = Vec::new();
    for (offset, op) in code.iter().copied().enumerate() {
        let instruction = u32::try_from(offset).unwrap_or(u32::MAX);
        match op {
            Op::Constant(index)
            | Op::GetGlobal(index)
            | Op::SetGlobal(index)
            | Op::GlobalIndexSet(index, _)
            | Op::Call(index, _)
            | Op::MakeClosure(index, _)
            | Op::FieldGet(index)
            | Op::FieldSet(index) => push_constant(&mut relocations, instruction, 0, index),
            Op::MakeRecord(index, _) => {
                push_constant(&mut relocations, instruction, 0, index);
            }
            Op::MakeEnum(type_index, variant_index, _)
            | Op::IsVariant(type_index, variant_index) => {
                push_constant(&mut relocations, instruction, 0, type_index);
                push_constant(&mut relocations, instruction, 1, variant_index);
            }
            Op::Jump(target) | Op::JumpIfFalse(target) | Op::JumpIfTrue(target) => {
                relocations.push(Relocation {
                    instruction,
                    kind: RelocationKind::CodeAddress { target },
                });
            }
            _ => {}
        }
    }
    relocations
}

fn push_constant(relocations: &mut Vec<Relocation>, instruction: u32, operand: u8, index: u16) {
    relocations.push(Relocation {
        instruction,
        kind: RelocationKind::Constant { operand, index },
    });
}
