//! Structural validation for complete executable bytecode images.

mod control_flow;

use std::fmt;

use crate::{Chunk, ChunkError, Intrinsic, Op};

use control_flow::validate_entry_control_flow;

/// Invalid executable bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableError {
    /// General chunk invariants are invalid.
    Chunk(ChunkError),
    /// No root instruction can terminate program execution.
    MissingEntryExit,
    /// Reachable initialization code falls past the end of the instruction stream.
    EntryFallthrough {
        /// Last reachable instruction before the fallthrough.
        instruction: usize,
    },
    /// Initialization control flow enters a callable body instead of calling it.
    EntryFunctionRegion {
        /// Instruction whose successor enters the callable body.
        instruction: usize,
        /// Callable-body instruction reached directly.
        target: usize,
    },
    /// An instruction references a missing constant.
    ConstantIndex {
        /// Instruction offset containing the operand.
        instruction: usize,
        /// Referenced constant-pool index.
        index: u16,
        /// Number of constants in the image.
        constants: usize,
    },
    /// An opcode name operand references a non-string constant.
    NameConstantType {
        /// Instruction offset containing the operand.
        instruction: usize,
        /// Referenced constant-pool index.
        index: u16,
        /// Semantic role of the name operand.
        operand: &'static str,
        /// Actual runtime value category.
        actual: &'static str,
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
    /// A named call or closure references no callable table entry.
    UnknownFunction {
        /// Instruction offset containing the reference.
        instruction: usize,
        /// Missing callable name.
        name: String,
    },
    /// A direct call's encoded arity differs from the callable table.
    FunctionArity {
        /// Instruction offset containing the call.
        instruction: usize,
        /// Referenced callable name.
        name: String,
        /// Arity declared by the callable table.
        expected: u8,
        /// Arity encoded in the call instruction.
        actual: u8,
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

/// Validate indices, targets, functions, intrinsics, and entry control flow in a complete image.
///
/// # Errors
///
/// Returns [`ExecutableError`] when the chunk violates an executable-image invariant.
pub fn validate_executable(chunk: &Chunk) -> Result<(), ExecutableError> {
    chunk
        .validate_invariants()
        .map_err(ExecutableError::Chunk)?;

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

    validate_entry_control_flow(chunk)
}

fn validate_instruction(chunk: &Chunk, instruction: usize, op: Op) -> Result<(), ExecutableError> {
    match op {
        Op::Constant(index) => validate_constant(chunk, instruction, index),
        Op::GetGlobal(index) | Op::SetGlobal(index) | Op::GlobalIndexSet(index, _) => {
            validate_name_constant(chunk, instruction, index, "global name").map(drop)
        }
        Op::Call(index, arity) => {
            let name = validate_name_constant(chunk, instruction, index, "function name")?;
            validate_function(chunk, instruction, name, Some(arity))
        }
        Op::MakeClosure(index, _) => {
            let name = validate_name_constant(chunk, instruction, index, "function name")?;
            validate_function(chunk, instruction, name, None)
        }
        Op::MakeRecord(index, _) => {
            validate_name_constant(chunk, instruction, index, "record type name").map(drop)
        }
        Op::FieldGet(index) | Op::FieldSet(index) => {
            validate_name_constant(chunk, instruction, index, "record field name").map(drop)
        }
        Op::MakeEnum(type_index, variant_index, _) | Op::IsVariant(type_index, variant_index) => {
            validate_name_constant(chunk, instruction, type_index, "enum type name")?;
            validate_name_constant(chunk, instruction, variant_index, "enum variant name")?;
            Ok(())
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

fn validate_name_constant<'a>(
    chunk: &'a Chunk,
    instruction: usize,
    index: u16,
    operand: &'static str,
) -> Result<&'a str, ExecutableError> {
    validate_constant(chunk, instruction, index)?;
    let value = &chunk.constants()[index as usize];
    match value {
        crate::Value::Str(name) => Ok(name),
        other => Err(ExecutableError::NameConstantType {
            instruction,
            index,
            operand,
            actual: other.type_name(),
        }),
    }
}

fn validate_function(
    chunk: &Chunk,
    instruction: usize,
    name: &str,
    actual_arity: Option<u8>,
) -> Result<(), ExecutableError> {
    let entry = chunk.functions().get(name).copied().or_else(|| {
        let canonical = name.to_ascii_lowercase();
        chunk.functions().get(&canonical).copied()
    });
    let Some((_, expected_arity)) = entry else {
        return Err(ExecutableError::UnknownFunction {
            instruction,
            name: name.to_string(),
        });
    };
    if let Some(actual) = actual_arity
        && actual != expected_arity
    {
        return Err(ExecutableError::FunctionArity {
            instruction,
            name: name.to_string(),
            expected: expected_arity,
            actual,
        });
    }
    Ok(())
}
