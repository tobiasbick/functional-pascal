//! Resolve routine metadata and captured bindings at their lexical declarations.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`

use super::{ScopeStack, canonical_symbol_name};
use fpas_lexer::Span;

impl ScopeStack {
    /// Refreshes live routine bindings after recursive capture metadata is completed.
    /// See `docs/pascal/language/functions/closures.md`.
    pub(crate) fn update_routine_capture_capabilities(
        &mut self,
        key: usize,
        task_bound: bool,
        task_free: bool,
    ) {
        for binding in self
            .scopes
            .iter_mut()
            .flat_map(|scope| scope.symbols.values_mut())
        {
            if binding.routine_capture_key == Some(key) {
                binding.symbol.task_bound = task_bound;
                binding.discard.value = task_free;
            }
        }
    }

    /// Associate a named routine's lexical binding with its AST capture metadata.
    pub(crate) fn set_routine_capture_key(&mut self, name: &str, key: usize) {
        let canonical = canonical_symbol_name(name);
        if let Some(binding) = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.symbols.get_mut(&canonical))
        {
            binding.routine_capture_key = Some(key);
        }
    }

    /// Resolve capture metadata at the visible binding, including name shadowing.
    pub(crate) fn routine_capture_key(&self, name: &str) -> Option<usize> {
        let canonical = canonical_symbol_name(name);
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(&canonical))
            .and_then(|binding| binding.routine_capture_key)
    }

    /// Find a capture's original scope even when a nearer binding shadows its name.
    pub(crate) fn capture_scope_index(&self, name: &str, declaration: Span) -> Option<usize> {
        let canonical = canonical_symbol_name(name);
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, scope)| {
                scope
                    .symbols
                    .get(&canonical)
                    .filter(|binding| binding.declaration == Some(declaration))
                    .map(|_| index)
            })
    }
}
