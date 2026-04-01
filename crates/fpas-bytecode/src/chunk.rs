use crate::op::Op;
use crate::value::Value;
use fpas_diagnostics::SourceLocation;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkError {
    InvalidInstructionOffset {
        offset: usize,
        code_len: usize,
    },
    InvalidJumpTarget {
        offset: usize,
        target: u32,
        code_len: usize,
    },
    NonJumpInstruction {
        offset: usize,
        opcode: String,
    },
    ConstantPoolOverflow {
        max_constants: usize,
    },
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInstructionOffset { offset, code_len } => {
                write!(
                    f,
                    "invalid instruction offset {offset}; chunk has {code_len} instructions"
                )
            }
            Self::InvalidJumpTarget {
                offset,
                target,
                code_len,
            } => {
                write!(
                    f,
                    "jump at offset {offset} targets {target}, but chunk currently has {code_len} instructions"
                )
            }
            Self::NonJumpInstruction { offset, opcode } => {
                write!(
                    f,
                    "instruction at offset {offset} is not a jump and cannot be patched: {opcode}"
                )
            }
            Self::ConstantPoolOverflow { max_constants } => {
                write!(f, "constant pool overflow: exceeds {max_constants} entries")
            }
        }
    }
}

impl std::error::Error for ChunkError {}

/// A compiled chunk of bytecode with its constant pool.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    /// Parallel to `code`: maps each instruction to a source location (1-based line and column).
    pub locations: Vec<SourceLocation>,
    /// Function table: name → (code_start, arity).
    pub functions: HashMap<String, (usize, u8)>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            locations: Vec::new(),
            functions: HashMap::new(),
        }
    }

    /// Emit an instruction, recording the source location.
    pub fn emit(&mut self, op: Op, location: SourceLocation) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        self.locations.push(location);
        idx
    }

    #[must_use]
    pub fn location_at(&self, instruction_index: usize) -> Option<SourceLocation> {
        self.locations.get(instruction_index).copied()
    }

    /// Add a constant to the pool, returning its index.
    pub fn add_constant(&mut self, value: Value) -> Result<u16, ChunkError> {
        // Reuse existing identical constant if present.
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return Ok(i as u16);
            }
        }
        if self.constants.len() >= u16::MAX as usize {
            return Err(ChunkError::ConstantPoolOverflow {
                max_constants: u16::MAX as usize,
            });
        }
        let idx = self.constants.len() as u16;
        self.constants.push(value);
        Ok(idx)
    }

    /// Patch a jump instruction at `offset` with the given target address.
    pub fn patch_jump(&mut self, offset: usize, target: u32) -> Result<(), ChunkError> {
        if target > self.code.len() as u32 {
            return Err(ChunkError::InvalidJumpTarget {
                offset,
                target,
                code_len: self.code.len(),
            });
        }

        match self.code.get_mut(offset) {
            Some(Op::Jump(addr) | Op::JumpIfFalse(addr) | Op::JumpIfTrue(addr)) => {
                *addr = target;
                Ok(())
            }
            Some(op) => Err(ChunkError::NonJumpInstruction {
                offset,
                opcode: format!("{op:?}"),
            }),
            None => Err(ChunkError::InvalidInstructionOffset {
                offset,
                code_len: self.code.len(),
            }),
        }
    }

    /// Current code length (next instruction offset).
    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn add_constant_reuses_existing_value() {
        let mut chunk = Chunk::new();

        let first = chunk.add_constant(Value::Integer(42));
        let second = chunk.add_constant(Value::Integer(42));

        assert_eq!(first, Ok(0));
        assert_eq!(second, Ok(0));
        assert_eq!(chunk.constants.len(), 1);
    }

    #[test]
    fn add_constant_returns_error_when_pool_limit_is_exceeded() {
        let mut chunk = Chunk::new();
        chunk.constants = (0..u16::MAX)
            .map(|value| Value::Integer(i64::from(value)))
            .collect();

        assert_eq!(
            chunk.add_constant(Value::Integer(i64::from(u16::MAX))),
            Err(ChunkError::ConstantPoolOverflow {
                max_constants: u16::MAX as usize,
            })
        );
    }

    #[test]
    fn patch_jump_rejects_target_past_chunk_end() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Jump(0), loc());

        assert_eq!(
            chunk.patch_jump(0, 2),
            Err(ChunkError::InvalidJumpTarget {
                offset: 0,
                target: 2,
                code_len: 1,
            })
        );
    }
}
