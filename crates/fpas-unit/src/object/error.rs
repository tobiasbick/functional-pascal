//! Structured register-object validation and codec errors.

use std::fmt;

use crate::object::{DefinitionTarget, SymbolKind};

/// Invalid relocatable register object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectError {
    /// Object schema version is incompatible.
    Version {
        /// Encoded version.
        actual: u16,
        /// Supported version.
        expected: u16,
    },
    /// Name is empty or not canonical lowercase ASCII.
    NonCanonicalName(String),
    /// A name is duplicated case-insensitively after canonicalization.
    DuplicateName(String),
    /// A canonically ordered table is not strictly increasing.
    NonDeterministicOrder(&'static str),
    /// A definition points outside its category table.
    InvalidDefinitionTarget(DefinitionTarget),
    /// An object-local reference points outside its category table.
    InvalidLocalReference {
        /// Required category.
        kind: SymbolKind,
        /// Invalid object-local index.
        index: u32,
    },
    /// An imported reference has the wrong category.
    ReferenceKind {
        /// Required category.
        expected: SymbolKind,
        /// Actual import category.
        actual: SymbolKind,
    },
    /// A table reference is invalid.
    InvalidTableReference(&'static str),
    /// Function contains no terminator or instruction.
    EmptyFunction {
        /// Function name.
        function: String,
    },
    /// Packed instruction is malformed.
    Instruction(String),
    /// Branch target precedes its function.
    BranchOutsideFunction {
        /// Absolute target.
        target: u32,
        /// Function start.
        function_start: u32,
    },
    /// A relocation target is invalid.
    InvalidRelocationTarget {
        /// Object-local function.
        function: u32,
        /// Function-local instruction.
        instruction: u32,
    },
    /// Two relocation records claim the same instruction.
    DuplicateRelocation {
        /// Object-local function.
        function: u32,
        /// Function-local instruction.
        instruction: u32,
    },
    /// Relocation coverage does not match the opcode operand.
    RelocationCoverage {
        /// Object-local function.
        function: u32,
        /// Function-local instruction.
        instruction: u32,
    },
    /// Sparse source run is unsorted or outside its function/source table.
    InvalidSourceRun {
        /// Function name.
        function: String,
        /// Function-local instruction.
        instruction: u32,
    },
    /// Fixed-width conversion overflowed.
    Overflow(&'static str),
    /// Deterministic object serialization failed.
    Encode(String),
    /// Object decoding failed.
    Decode(String),
    /// Encoded object exceeds the sidecar payload limit.
    PayloadSize {
        /// Encoded size.
        size: usize,
        /// Maximum accepted size.
        maximum: usize,
    },
}

impl fmt::Display for ObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid register object: {self:?}")
    }
}

impl std::error::Error for ObjectError {}
