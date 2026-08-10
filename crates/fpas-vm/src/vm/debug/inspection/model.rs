//! Public debugger inspection records and resource limits.

use super::super::types::SourceLocation;

/// Limits applied to one stopped-state inspection snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugInspectionLimits {
    /// Largest accepted frame page and retained frame count.
    pub max_frames: usize,
    /// Largest variable page and children retained per value.
    pub max_children: usize,
    /// Largest rendered Unicode scalar count for one string.
    pub max_string_chars: usize,
    /// Largest recursively expandable value depth.
    pub max_depth: usize,
    /// Largest number of stable variables references per stop.
    pub max_handles: usize,
    /// Largest cumulative UTF-8 bytes returned by one variables request.
    pub max_output_bytes: usize,
}

impl Default for DebugInspectionLimits {
    fn default() -> Self {
        Self {
            max_frames: 256,
            max_children: 256,
            max_string_chars: 4_096,
            max_depth: 16,
            max_handles: 16_384,
            max_output_bytes: 1024 * 1024,
        }
    }
}

/// One bounded page and the complete available item count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paginated<T> {
    /// Requested page items after configured bounds were applied.
    pub items: Vec<T>,
    /// Complete number of items available at this stop.
    pub total: usize,
}

/// One logical FPAS call frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugFrame {
    /// Stable stop-local frame identifier.
    pub id: u64,
    /// Canonical function name.
    pub name: String,
    /// Source position for the active boundary or saved call site.
    pub location: Option<SourceLocation>,
    /// Zero-based depth, with zero representing the active frame.
    pub depth: usize,
}

/// Source-level scope category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugScopeKind {
    /// Explicit function or procedure parameters.
    Parameters,
    /// Lexically visible local bindings.
    Locals,
    /// Values captured from enclosing routines.
    Captures,
    /// Program and unit globals.
    Globals,
}

/// One source-level scope for a stopped frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugScope {
    /// Human-readable scope name.
    pub name: String,
    /// Stable scope category.
    pub kind: DebugScopeKind,
    /// Stop-local reference accepted by `variables`.
    pub variables_reference: u64,
    /// Number of immediately named variables.
    pub named_variables: usize,
    /// Globals can be more expensive than frame-local scopes.
    pub expensive: bool,
}

/// One bounded debugger variable or aggregate child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugVariable {
    /// Source binding, field, index, key, or wrapper child name.
    pub name: String,
    /// Bounded non-recursive value summary.
    pub value: String,
    /// Portable source or runtime type name.
    pub type_name: String,
    /// Stop-local reference for lazy child expansion, or zero for a leaf.
    pub variables_reference: u64,
    /// Number of named children.
    pub named_variables: usize,
    /// Number of indexed children.
    pub indexed_variables: usize,
    /// Optional stable presentation hint such as `captured mutable` or `cycle`.
    pub presentation_hint: Option<String>,
}
