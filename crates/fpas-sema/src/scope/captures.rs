//! Resolve routine metadata and captured bindings at their lexical declarations.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`

use super::{ScopeStack, canonical_symbol_name};
use fpas_lexer::Span;

impl ScopeStack {
    /// Collect mutable storage roots before entering a separately analyzed routine.
    pub(crate) fn mutable_bindings(&self) -> Vec<(String, Span, crate::types::Ty)> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.symbols.values())
            .filter_map(|binding| {
                if !binding.symbol.mutable {
                    return None;
                }
                binding.declaration.map(|span| {
                    (
                        binding.original_name.clone(),
                        span,
                        binding.symbol.ty.clone(),
                    )
                })
            })
            .collect()
    }

    /// Globals can be written by ordinary source routines without lexical captures.
    pub(crate) fn mutable_global_bindings(&self) -> Vec<(String, Span, crate::types::Ty)> {
        self.scopes[0]
            .symbols
            .values()
            .filter_map(|binding| {
                if !binding.symbol.mutable {
                    return None;
                }
                binding.declaration.map(|span| {
                    (
                        binding.original_name.clone(),
                        span,
                        binding.symbol.ty.clone(),
                    )
                })
            })
            .collect()
    }

    /// Update an exact source binding even when a nearer declaration shadows its name.
    pub(crate) fn set_capture_capability(
        &mut self,
        name: &str,
        declaration: Span,
        capability: crate::TransferCapability,
        join: bool,
    ) {
        let name = canonical_symbol_name(name);
        for scope in &mut self.scopes {
            if let Some(binding) = scope.symbols.get_mut(&name)
                && binding.declaration == Some(declaration)
            {
                binding.symbol.capability = if join {
                    binding.symbol.capability.join(capability)
                } else {
                    capability
                };
                break;
            }
        }
    }

    /// Refreshes live routine bindings after recursive capture metadata is completed.
    /// See `docs/pascal/language/functions/closures.md`.
    pub(crate) fn update_routine_capture_capabilities(
        &mut self,
        key: usize,
        capability: crate::TransferCapability,
    ) {
        for binding in self
            .scopes
            .iter_mut()
            .flat_map(|scope| scope.symbols.values_mut())
        {
            if binding.routine_capture_key == Some(key) {
                binding.symbol.capability = capability;
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
