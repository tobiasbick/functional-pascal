//! Lexical visibility in routine bodies versus ordered executable value use.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::NavigationDocument;
use crate::{DocumentSymbol, SymbolKind};

impl NavigationDocument {
    /// Resolve later enclosing bindings in routine bodies without exposing later body locals.
    pub(crate) fn symbol_visible_at(&self, symbol: &DocumentSymbol, offset: usize) -> bool {
        if symbol.is_visible_at(offset) {
            return true;
        }
        if !matches!(symbol.kind, SymbolKind::Constant | SymbolKind::Variable)
            || !(symbol.scope_span.offset() <= offset && offset < symbol.scope_span.end())
        {
            return false;
        }
        if let Some(semantic) = self
            .analysis
            .as_ref()
            .and_then(|analysis| analysis.semantic())
            && self
                .tokens
                .iter()
                .filter(|token| {
                    token.span.offset <= offset && offset <= token.span.offset + token.span.length
                })
                .any(|token| {
                    semantic
                        .metadata()
                        .binding_references
                        .get(&(token.span.source_id, token.span.offset))
                        == Some(&(symbol.full_span.source_id(), symbol.full_span.offset()))
                })
        {
            return true;
        }
        self.all_symbols()
            .into_iter()
            .filter(|routine| {
                matches!(
                    routine.kind,
                    SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method
                )
            })
            .filter(|routine| {
                routine.full_span.offset() <= offset && offset < routine.full_span.end()
            })
            .min_by_key(|routine| routine.full_span.length())
            .is_some_and(|routine| {
                symbol.scope_span.offset() <= routine.full_span.offset()
                    && routine.full_span.end() <= symbol.scope_span.end()
                    && symbol.scope_span != routine.full_span
            })
    }
}
