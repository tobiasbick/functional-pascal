//! Free-variable capture analysis for closures.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use std::collections::{HashMap, HashSet};

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

/// Lexical captures and recursive routine dependencies still being analyzed.
/// See `docs/pascal/language/functions/var-parameters.md`.
pub struct CaptureAnalysis {
    /// Lexical scope boundary: declarations in this scope belong to the routine itself.
    pub scope_index: usize,
    /// Enclosing bindings whose capture information is already available.
    pub captures: Vec<CaptureBinding>,
    /// Declaration identities of active routines needed by this body.
    pub pending_routines: Vec<usize>,
}

/// Collect lexical captures referenced by `body`.
///
/// A name is captured when it resolves to a `Const`, `Var`, `Param`, or `ForVar` in a non-root scope
/// outside the closure's own scope frame (`closure_scope_index`). Captures required only by a
/// nested closure or named routine are propagated from that routine's analyzed metadata, which
/// preserves its own parameter and local shadowing. Named sibling references propagate the same
/// metadata, retaining the captured declaration rather than resolving its name again.
/// Active recursive targets retain declaration-key dependencies for deferred lifetime checks.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[must_use]
pub fn collect_captures(
    scopes: &ScopeStack,
    closure_scope_index: usize,
    body: &FuncBody,
    closure_infos: &ClosureInfoMap,
    nested_routine_captures: &NestedRoutineCaptureMap,
    pending_routine_captures: &HashMap<usize, Vec<usize>>,
    pending_closure_captures: &HashMap<usize, Vec<usize>>,
) -> CaptureAnalysis {
    let mut collector = CaptureCollector {
        scopes,
        closure_scope_index,
        closure_infos,
        nested_routine_captures,
        pending_routine_captures,
        pending_closure_captures,
        pending_routines: Vec::new(),
        captures: Vec::new(),
        seen_routines: HashSet::new(),
        bound_scopes: Vec::new(),
    };
    collector.collect_from_body(body);
    // Name lookup in the lowered frame must prefer the nearest captured declaration.
    collector
        .captures
        .sort_by_key(|capture| scopes.capture_scope_index(&capture.name, capture.declaration));
    collector.pending_routines.sort_unstable();
    collector.pending_routines.dedup();
    CaptureAnalysis {
        scope_index: closure_scope_index,
        captures: collector.captures,
        pending_routines: collector.pending_routines,
    }
}

struct CaptureCollector<'a> {
    scopes: &'a ScopeStack,
    closure_scope_index: usize,
    closure_infos: &'a ClosureInfoMap,
    nested_routine_captures: &'a NestedRoutineCaptureMap,
    pending_routine_captures: &'a HashMap<usize, Vec<usize>>,
    pending_closure_captures: &'a HashMap<usize, Vec<usize>>,
    pending_routines: Vec<usize>,
    captures: Vec<CaptureBinding>,
    seen_routines: HashSet<usize>,
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
        let Some((scope_index, symbol, declaration)) =
            self.scopes.lookup_with_scope_and_declaration(name)
        else {
            return;
        };
        if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure) {
            if let Some(key) = self.scopes.routine_capture_key(name) {
                self.collect_routine_captures(key);
            }
            return;
        }
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
        if self.captures.iter().any(|capture| {
            capture.declaration == declaration && capture.name.eq_ignore_ascii_case(name)
        }) {
            return;
        }
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

impl CaptureCollector<'_> {
    fn collect_routine_captures(&mut self, key: usize) {
        if !self.seen_routines.insert(key) {
            return;
        }
        if let Some(info) = self.nested_routine_captures.get(&key) {
            self.collect_transitive_captures(&info.captures.clone());
            if let Some(pending) = self.pending_routine_captures.get(&key) {
                for &key in pending {
                    self.collect_routine_captures(key);
                }
            }
        } else {
            self.pending_routines.push(key);
        }
    }
}
