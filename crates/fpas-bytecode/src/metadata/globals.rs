//! Dense global-slot declarations.

use crate::{DebugTypeId, FunctionId, InstructionAddress, StringId};

/// Exact executable identity of one global source initializer store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalInitializer {
    /// Function containing the store.
    pub function: FunctionId,
    /// Exact store instruction.
    pub instruction: InstructionAddress,
}

/// Metadata for one executable-wide global slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalInfo {
    /// Canonical diagnostic name in the string table.
    pub name: StringId,
    /// Machine-readable type of values stored in this slot.
    pub ty: DebugTypeId,
    /// Whether bytecode may store a new value after initialization.
    pub mutable: bool,
    /// Exact source-declaration store, when present in this executable.
    pub initializer: Option<GlobalInitializer>,
}
