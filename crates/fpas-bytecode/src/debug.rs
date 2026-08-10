//! Portable source-level debugger metadata.

use crate::{InstructionAddress, Register, SourceId, StringId};

/// Complete debugger metadata for one executable function.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionDebugInfo {
    /// Lexical scope tree in dense identifier order.
    pub scopes: Vec<DebugScope>,
    /// Source-visible register-backed bindings.
    pub bindings: Vec<DebugBinding>,
    /// Ordered executable sequence points.
    pub sequence_points: Vec<SequencePoint>,
}

/// One function-local lexical scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugScope {
    /// Dense function-local scope identifier.
    pub id: u32,
    /// Parent scope, absent only for the root scope.
    pub parent: Option<u32>,
}

/// Source-level role of a register-backed binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugBindingKind {
    /// Explicit routine parameter.
    Parameter,
    /// Lexically declared local variable.
    Local,
    /// Value captured from an enclosing routine.
    Capture,
}

/// One source position stored in debugger metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSourceLocation {
    /// Source-path table identifier.
    pub source: SourceId,
    /// One-based line.
    pub line: u32,
    /// One-based column.
    pub column: u32,
}

/// A source-visible binding mapped to its fixed register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugBinding {
    /// Source name in the executable string table.
    pub name: StringId,
    /// Display type in the executable string table.
    pub type_name: StringId,
    /// Register relative to the owning frame base.
    pub register: Register,
    /// Source-level role.
    pub kind: DebugBindingKind,
    /// Whether source semantics permit reassignment.
    pub mutable: bool,
    /// Lexical scope containing the binding.
    pub scope: u32,
    /// Declaration location when source syntax supplied one.
    pub declaration: Option<DebugSourceLocation>,
    /// Whether the binding is compiler-generated and hidden from normal scopes.
    pub hidden: bool,
    /// Whether the stored value is a mutable closure cell to dereference for display.
    pub cell_backed: bool,
}

/// A source execution boundary at one global instruction address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencePoint {
    /// Instruction that begins this source operation.
    pub instruction: InstructionAddress,
    /// Source location represented by the point.
    pub location: DebugSourceLocation,
    /// Innermost lexical scope active at the point.
    pub scope: u32,
}
