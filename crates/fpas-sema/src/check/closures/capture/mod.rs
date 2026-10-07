//! Free-variable capture analysis for closures.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use std::collections::HashSet;

use crate::scope::{ScopeStack, SymbolKind};
use crate::types::Ty;
use fpas_parser::FuncBody;

mod traversal;

use super::{ClosureInfoMap, NestedRoutineCaptureMap};

/// A free variable captured by a closure from an enclosing scope.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureBinding {
    /// Whether the binding's reachable captures are statically task-free.
    pub task_free: bool,
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
    /// `true` when the binding is a `var` parameter whose storage is the caller's variable.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    pub reference: bool,
}

/// Collect lexical captures referenced by `body`.
///
/// A name is captured when it resolves to a `Const`, `Var`, `Param`, or `ForVar` in a non-root scope
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

impl CaptureCollector<'_> {
    fn consider_name(&mut self, name: &str) {
        let canonical = name.to_ascii_lowercase();
        if self
            .bound_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(&canonical))
        {
            return;
        }
        if !self.seen.insert(canonical) {
            return;
        }
        let Some((scope_index, symbol, declaration)) =
            self.scopes.lookup_with_scope_and_declaration(name)
        else {
            return;
        };
        if scope_index == 0 || scope_index >= self.closure_scope_index {
            return;
        }
        if !matches!(
            symbol.kind,
            SymbolKind::Const | SymbolKind::Var | SymbolKind::Param | SymbolKind::ForVar
        ) {
            return;
        }
        let Some(declaration) = declaration else {
            return;
        };
        self.captures.push(CaptureBinding {
            task_free: self.scopes.discard_info(name).value,
            name: name.to_string(),
            ty: symbol.ty.clone(),
            mutable: symbol.mutable,
            task_bound: symbol.task_bound,
            declaration,
            reference: symbol.is_var_parameter(),
        });
    }
}
