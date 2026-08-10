//! Source-level debugger metadata retained beside operational IR.

use crate::{BlockId, LocalId, SourceSpan, TypeId};

/// Complete source-debug metadata for one function.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionDebugInfo {
    /// Lexical scope tree in dense identifier order.
    pub scopes: Vec<DebugScope>,
    /// Source-visible bindings owned by the function.
    pub bindings: Vec<DebugBinding>,
    /// Source execution points attached to IR instructions.
    pub sequence_points: Vec<SequencePoint>,
}

/// One lexical source scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugScope {
    /// Dense function-local scope identifier.
    pub id: u32,
    /// Parent scope, absent only for the function root.
    pub parent: Option<u32>,
}

/// Source-level role of a debug binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugBindingKind {
    /// Explicit routine parameter.
    Parameter,
    /// Lexically declared local variable.
    Local,
    /// Value captured from an enclosing routine.
    Capture,
}

/// A source-visible name backed by one fixed function-local register allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugBinding {
    /// Local storage identifier allocated by lowering.
    pub local: LocalId,
    /// Canonical source name.
    pub name: String,
    /// Source-level role.
    pub kind: DebugBindingKind,
    /// Lowered value type.
    pub ty: TypeId,
    /// Whether source semantics permit reassignment.
    pub mutable: bool,
    /// Lexical scope containing the binding.
    pub scope: u32,
    /// Declaration location when one is available from source syntax.
    pub declaration: Option<SourceSpan>,
    /// Whether this is compiler-generated storage omitted from normal debugger scopes.
    pub hidden: bool,
    /// Whether the register contains a mutable capture cell rather than its displayed value.
    pub cell_backed: bool,
}

/// A debugger execution boundary attached to one source-bearing IR instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencePoint {
    /// Owning basic block.
    pub block: BlockId,
    /// Zero-based instruction index within the block.
    pub instruction: usize,
    /// Source range represented by the point.
    pub source: SourceSpan,
    /// Innermost lexical scope active at the point.
    pub scope: u32,
}
