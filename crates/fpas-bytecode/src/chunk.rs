use crate::op::Op;
use crate::value::Value;
use fpas_diagnostics::SourceLocation;
use std::collections::HashMap;
use std::fmt;

/// Largest valid constant-pool index when the pool holds the maximum number of entries.
pub const MAX_CONSTANT_INDEX: u16 = u16::MAX - 1;

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
        opcode: Op,
    },
    ConstantPoolOverflow {
        max_constants: usize,
    },
    CodeLocationLengthMismatch {
        code_len: usize,
        locations_len: usize,
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
                    "instruction at offset {offset} is not a jump and cannot be patched: {opcode:?}"
                )
            }
            Self::ConstantPoolOverflow { max_constants } => {
                write!(f, "constant pool overflow: exceeds {max_constants} entries")
            }
            Self::CodeLocationLengthMismatch {
                code_len,
                locations_len,
            } => {
                write!(
                    f,
                    "chunk invariant violated: code has {code_len} instructions but locations has {locations_len} entries"
                )
            }
        }
    }
}

impl std::error::Error for ChunkError {}

/// A compiled chunk of bytecode with its constant pool.
#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<Op>,
    constants: Vec<Value>,
    /// Parallel to `code`: maps each instruction to a source location (1-based line and column).
    locations: Vec<SourceLocation>,
    /// Function table: name → (code_start, arity).
    functions: HashMap<String, (usize, u8)>,
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

    /// Instruction stream.
    #[must_use]
    pub fn code(&self) -> &[Op] {
        &self.code
    }

    /// Constant pool.
    #[must_use]
    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    /// Source location per instruction (parallel to [`Self::code`]).
    #[must_use]
    pub fn locations(&self) -> &[SourceLocation] {
        &self.locations
    }

    /// Function entry points keyed by canonical name.
    #[must_use]
    pub fn functions(&self) -> &HashMap<String, (usize, u8)> {
        &self.functions
    }

    /// Register a callable entry point in the function table.
    pub fn insert_function(&mut self, name: impl Into<String>, code_start: usize, arity: u8) {
        self.functions.insert(name.into(), (code_start, arity));
    }

    /// Returns `true` when `idx` refers to an existing constant-pool entry.
    #[must_use]
    pub fn is_valid_constant_index(&self, idx: u16) -> bool {
        (idx as usize) < self.constants.len()
    }

    /// Verify structural invariants (`code`/`locations` length parity).
    pub fn validate_invariants(&self) -> Result<(), ChunkError> {
        if self.code.len() != self.locations.len() {
            return Err(ChunkError::CodeLocationLengthMismatch {
                code_len: self.code.len(),
                locations_len: self.locations.len(),
            });
        }
        Ok(())
    }

    /// Emit an instruction, recording the source location.
    pub fn emit(&mut self, op: Op, location: SourceLocation) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        self.locations.push(location);
        debug_assert_eq!(self.code.len(), self.locations.len());
        idx
    }

    #[must_use]
    pub fn location_at(&self, instruction_index: usize) -> Option<SourceLocation> {
        self.locations.get(instruction_index).copied()
    }

    /// Add a constant to the pool, returning its index.
    pub fn add_constant(&mut self, value: Value) -> Result<u16, ChunkError> {
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return Ok(i as u16);
            }
        }
        if self.constants.len() > MAX_CONSTANT_INDEX as usize {
            return Err(ChunkError::ConstantPoolOverflow {
                max_constants: MAX_CONSTANT_INDEX as usize + 1,
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
                opcode: *op,
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

    /// Returns `true` when the chunk contains no instructions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Returns `true` if this chunk may enqueue parallel tasks (`go` / detached spawn).
    ///
    /// The scan is purely static over [`Op::SpawnTask`] / [`Op::SpawnDetachedTask`] in `code`;
    /// [`Op::Yield`] and other opcodes do not affect the result.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 1), `docs/pascal/language/concurrency/README.md`
    #[must_use]
    pub fn uses_spawn_tasks(&self) -> bool {
        self.code
            .iter()
            .any(|op| matches!(op, Op::SpawnTask(_) | Op::SpawnDetachedTask(_)))
    }

    /// Pre-seed the constant pool in unit tests.
    #[cfg(test)]
    pub(crate) fn set_constants_for_test(&mut self, constants: Vec<Value>) {
        self.constants = constants;
    }

    /// Push an instruction without a location entry (corrupts invariants).
    #[cfg(test)]
    pub(crate) fn push_code_without_location_for_test(&mut self, op: Op) {
        self.code.push(op);
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
        assert_eq!(chunk.constants().len(), 1);
    }

    #[test]
    fn add_constant_reuses_nan_with_same_payload() {
        let mut chunk = Chunk::new();
        let nan = Value::Real(f64::from_bits(0x7FF8_0000_0000_0001));

        let first = chunk.add_constant(nan.clone());
        let second = chunk.add_constant(nan);

        assert_eq!(first, Ok(0));
        assert_eq!(second, Ok(0));
        assert_eq!(chunk.constants().len(), 1);
    }

    #[test]
    fn add_constant_returns_error_when_pool_limit_is_exceeded() {
        let mut chunk = Chunk::new();
        chunk.set_constants_for_test(
            (0..=MAX_CONSTANT_INDEX)
                .map(|value| Value::Integer(i64::from(value)))
                .collect(),
        );

        assert_eq!(
            chunk.add_constant(Value::Integer(i64::from(u16::MAX))),
            Err(ChunkError::ConstantPoolOverflow {
                max_constants: MAX_CONSTANT_INDEX as usize + 1,
            })
        );
    }

    #[test]
    fn patch_jump_updates_jump_target() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Jump(0), loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(chunk.patch_jump(0, 1), Ok(()));
        assert_eq!(chunk.code(), &[Op::Jump(1), Op::Halt]);
    }

    #[test]
    fn patch_jump_updates_conditional_targets() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::JumpIfFalse(0), loc());
        chunk.emit(Op::JumpIfTrue(0), loc());
        chunk.emit(Op::Halt, loc());

        assert_eq!(chunk.patch_jump(0, 2), Ok(()));
        assert_eq!(chunk.patch_jump(1, 3), Ok(()));
        assert_eq!(
            chunk.code(),
            &[Op::JumpIfFalse(2), Op::JumpIfTrue(3), Op::Halt]
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

    #[test]
    fn patch_jump_rejects_non_jump_instruction() {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Pop, loc());

        assert_eq!(
            chunk.patch_jump(0, 0),
            Err(ChunkError::NonJumpInstruction {
                offset: 0,
                opcode: Op::Pop,
            })
        );
    }

    #[test]
    fn emit_keeps_code_and_locations_aligned() {
        let mut chunk = Chunk::new();
        let first = loc();
        let second = SourceLocation::new(2, 3);

        chunk.emit(Op::Unit, first);
        chunk.emit(Op::Halt, second);

        assert_eq!(chunk.location_at(0), Some(first));
        assert_eq!(chunk.location_at(1), Some(second));
        assert!(chunk.validate_invariants().is_ok());
    }

    #[test]
    fn validate_invariants_detects_length_mismatch() {
        let mut chunk = Chunk::new();
        chunk.push_code_without_location_for_test(Op::Halt);

        assert_eq!(
            chunk.validate_invariants(),
            Err(ChunkError::CodeLocationLengthMismatch {
                code_len: 1,
                locations_len: 0,
            })
        );
    }

    #[test]
    fn is_valid_constant_index_tracks_pool_size() -> Result<(), ChunkError> {
        let mut chunk = Chunk::new();
        assert!(!chunk.is_valid_constant_index(0));

        let idx = chunk.add_constant(Value::Integer(1))?;
        assert_eq!(idx, 0);
        assert!(chunk.is_valid_constant_index(0));
        assert!(!chunk.is_valid_constant_index(1));
        Ok(())
    }
}
