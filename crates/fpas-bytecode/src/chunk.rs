use crate::op::Op;
use crate::value::Value;
use fpas_diagnostics::SourceLocation;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkError {
    InvalidInstructionOffset { offset: usize, code_len: usize },
    NonJumpInstruction { offset: usize, opcode: String },
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
            Self::NonJumpInstruction { offset, opcode } => {
                write!(
                    f,
                    "instruction at offset {offset} is not a jump and cannot be patched: {opcode}"
                )
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
    ///
    /// # Panics
    ///
    /// Panics if the constant pool exceeds 65 535 entries.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        // Reuse existing identical constant if present.
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return i as u16;
            }
        }
        assert!(
            self.constants.len() < u16::MAX as usize,
            "constant pool overflow: exceeds {} entries",
            u16::MAX
        );
        let idx = self.constants.len() as u16;
        self.constants.push(value);
        idx
    }

    /// Patch a jump instruction at `offset` with the given target address.
    pub fn patch_jump(&mut self, offset: usize, target: u32) -> Result<(), ChunkError> {
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
