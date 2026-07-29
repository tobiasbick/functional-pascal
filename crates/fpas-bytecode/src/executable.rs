//! Structural validation for complete executable bytecode images.

use std::fmt;

use crate::{Chunk, ChunkError, Intrinsic, Op};

/// Invalid executable bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableError {
    /// General chunk invariants are invalid.
    Chunk(ChunkError),
    /// No halt instruction can terminate program initialization.
    MissingHalt,
    /// An instruction references a missing constant.
    ConstantIndex {
        /// Instruction offset containing the operand.
        instruction: usize,
        /// Referenced constant-pool index.
        index: u16,
        /// Number of constants in the image.
        constants: usize,
    },
    /// A control-flow instruction targets outside the image.
    CodeTarget {
        /// Instruction offset containing the operand.
        instruction: usize,
        /// Referenced absolute instruction offset.
        target: u32,
        /// Number of instructions in the image.
        code: usize,
    },
    /// A callable entry points outside the image.
    FunctionOffset {
        /// Canonical callable name.
        name: String,
        /// Invalid instruction offset.
        offset: usize,
        /// Number of instructions in the image.
        code: usize,
    },
    /// An intrinsic opcode contains an unknown wire identifier.
    Intrinsic {
        /// Instruction offset containing the operand.
        instruction: usize,
        /// Unknown intrinsic identifier.
        intrinsic: u16,
    },
}

impl fmt::Display for ExecutableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid executable bytecode: {self:?}")
    }
}

impl std::error::Error for ExecutableError {}

/// Validate indices, targets, functions, intrinsics, and termination in a complete image.
pub fn validate_executable(chunk: &Chunk) -> Result<(), ExecutableError> {
    chunk
        .validate_invariants()
        .map_err(ExecutableError::Chunk)?;

    if !chunk.code().iter().any(|op| matches!(op, Op::Halt)) {
        return Err(ExecutableError::MissingHalt);
    }

    for (name, (offset, _)) in chunk.functions() {
        if *offset >= chunk.len() {
            return Err(ExecutableError::FunctionOffset {
                name: name.clone(),
                offset: *offset,
                code: chunk.len(),
            });
        }
    }

    for (instruction, op) in chunk.code().iter().copied().enumerate() {
        validate_instruction(chunk, instruction, op)?;
    }

    Ok(())
}

fn validate_instruction(chunk: &Chunk, instruction: usize, op: Op) -> Result<(), ExecutableError> {
    match op {
        Op::Constant(index)
        | Op::GetGlobal(index)
        | Op::SetGlobal(index)
        | Op::GlobalIndexSet(index, _)
        | Op::Call(index, _)
        | Op::MakeClosure(index, _)
        | Op::MakeRecord(index, _)
        | Op::FieldGet(index)
        | Op::FieldSet(index) => validate_constant(chunk, instruction, index),
        Op::MakeEnum(type_index, variant_index, _) | Op::IsVariant(type_index, variant_index) => {
            validate_constant(chunk, instruction, type_index)?;
            validate_constant(chunk, instruction, variant_index)
        }
        Op::Jump(target)
        | Op::JumpIfFalse(target)
        | Op::JumpIfTrue(target)
        | Op::JumpIfLocalGt(_, _, target)
        | Op::JumpIfLocalLt(_, _, target) => {
            if target as usize >= chunk.len() {
                return Err(ExecutableError::CodeTarget {
                    instruction,
                    target,
                    code: chunk.len(),
                });
            }
            Ok(())
        }
        Op::Intrinsic(intrinsic) if Intrinsic::from_u16(intrinsic).is_none() => {
            Err(ExecutableError::Intrinsic {
                instruction,
                intrinsic,
            })
        }
        _ => Ok(()),
    }
}

fn validate_constant(chunk: &Chunk, instruction: usize, index: u16) -> Result<(), ExecutableError> {
    if chunk.is_valid_constant_index(index) {
        return Ok(());
    }
    Err(ExecutableError::ConstantIndex {
        instruction,
        index,
        constants: chunk.constants().len(),
    })
}
