//! Define read-only pattern bindings with the matched value's capture metadata.
//!
//! Documentation: `docs/pascal/language/pattern-matching/syntax.md`,
//! `docs/pascal/language/functions/discard.md`

use super::super::super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Define one pattern binding without losing task-bound state or discard proofs.
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
        let task_bound = self.type_can_contain_callable(ty)
            && self.expr_is_task_bound(Self::expr_lookup_key(value));
        let defined = self.scopes.define_with_declaration(
            name,
            Symbol {
                constant: None,
                ty: ty.clone(),
                mutable: false,
                kind: SymbolKind::Var,
                task_bound,
            },
            span,
        );
        if defined {
            self.record_binding_discard_info(name, ty, false, Some(value));
        }
        defined
    }
}
