//! Free-variable capture analysis for closures.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use std::collections::HashSet;

use crate::scope::{ScopeStack, SymbolKind};
use crate::types::Ty;
use fpas_parser::{Decl, Designator, DesignatorPart, Expr, FuncBody, PostfixOperation, Stmt};

use super::{ClosureInfoMap, NestedRoutineCaptureMap};

/// A free variable captured by a closure from an enclosing scope.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureBinding {
    /// Source name of the captured binding.
    pub name: String,
    /// Resolved semantic type at the capture boundary.
    pub ty: Ty,
    /// `true` when the capture is mutable (cell-backed at runtime).
    pub mutable: bool,
    /// `true` when the captured binding already holds a task-bound value
    /// (for example a nested closure that captured a mutable cell).
    pub task_bound: bool,
    /// Exact declaration of the captured source binding.
    pub declaration: fpas_lexer::Span,
}

/// Collect lexical captures referenced by `body`.
///
/// A name is captured when it resolves to local storage or a named routine in a non-root scope
/// outside the closure's own scope frame (`closure_scope_index`). Captures required only by a
/// nested closure or named routine are propagated from that routine's analyzed metadata, which
/// preserves its own parameter and local shadowing.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[must_use]
pub fn collect_captures(
    scopes: &ScopeStack,
    closure_scope_index: usize,
    body: &FuncBody,
    closure_infos: &ClosureInfoMap,
    nested_routine_captures: &NestedRoutineCaptureMap,
) -> Vec<CaptureBinding> {
    let mut collector = CaptureCollector {
        scopes,
        closure_scope_index,
        closure_infos,
        nested_routine_captures,
        captures: Vec::new(),
        seen: HashSet::new(),
        bound_scopes: Vec::new(),
    };
    collector.collect_from_body(body);
    collector.captures
}

struct CaptureCollector<'a> {
    scopes: &'a ScopeStack,
    closure_scope_index: usize,
    closure_infos: &'a ClosureInfoMap,
    nested_routine_captures: &'a NestedRoutineCaptureMap,
    captures: Vec<CaptureBinding>,
    seen: HashSet<String>,
    bound_scopes: Vec<HashSet<String>>,
}

mod traversal;
