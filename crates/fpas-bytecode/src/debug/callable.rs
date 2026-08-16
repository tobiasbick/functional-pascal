//! Source bindings, scopes, and sequence points used by debugger adapters.

use crate::{
    DebugBindingId, DebugTypeId, FunctionId, InstructionAddress, Register, SourceId, StringId,
};

/// Complete debugger metadata for one executable function.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionDebugInfo {
    /// Lexical scope tree in dense identifier order.
    pub scopes: Vec<DebugScope>,
    /// Source-visible register-backed bindings.
    pub bindings: Vec<DebugBinding>,
    /// Ordered executable sequence points.
    pub sequence_points: Vec<SequencePoint>,
    /// Portable result type when debugger metadata is available.
    ///
    /// `None` means the function has no retained result metadata, not `unit` and not
    /// `Dynamic`. **Documentation:** `docs/pascal/tools/debugger.md`
    pub result_type: Option<DebugTypeId>,
    /// Lexical owner of a capturing function; absent when the function has no captures.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub lexical_owner: Option<FunctionId>,
    /// Capture identity in runtime closure ABI order.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub capture_sources: Vec<DebugCaptureSource>,
}

/// Capture representation recorded for debugger construction and verification.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugCaptureKind {
    /// The closure captures an immutable value.
    Value,
    /// The closure captures a mutable cell.
    Cell,
    /// The closure reuses an enclosing mutable cell.
    EnclosingCell,
}

/// One exact owner binding captured by a nested function, in closure ABI order.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugCaptureSource {
    /// Dense owner-function binding identity. Runtime names are not a substitute.
    pub binding: DebugBindingId,
    /// Portable type of the captured value.
    pub ty: DebugTypeId,
    /// Representation mandated by semantic capture analysis.
    pub kind: DebugCaptureKind,
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
    /// Machine-readable type in the executable debug-type table.
    pub ty: DebugTypeId,
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
    /// Exact executable instruction that performs the source-declaration store.
    pub initializer: Option<InstructionAddress>,
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
