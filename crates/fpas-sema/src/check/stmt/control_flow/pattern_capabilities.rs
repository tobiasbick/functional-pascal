//! Define read-only pattern bindings with the matched value's capture metadata.
//!
//! Documentation: `docs/pascal/language/pattern-matching/syntax.md`,
//! `docs/pascal/language/functions/discard.md`

use super::super::super::Checker;
use crate::TransferCapability;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Define one pattern binding without losing task-bound capture state.
    ///
    /// Capture-free scalar types keep their own type-based guarantees. Callable
    /// contents inherit the matched expression's conservative capture metadata.
    pub(super) fn define_pattern_binding(
        &mut self,
        name: &str,
        ty: &Ty,
        value: &Expr,
        span: Span,
    ) -> bool {
        let capability = if self.type_can_contain_callable(ty) {
            self.expr_capability(Self::expr_lookup_key(value))
        } else {
            TransferCapability::Transferable
        };
        let defined = self.scopes.define_with_declaration(
            name,
            Symbol {
                constant: None,
                ty: ty.clone(),
                mutable: false,
                kind: SymbolKind::Var,
                capability,
            },
            span,
        );
        if defined {
            self.record_pending_capture_binding(name, Some(value));
        }
        defined
    }
}
