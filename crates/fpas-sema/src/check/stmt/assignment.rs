//! Assignment statement checking.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`

use super::Checker;
use crate::scope::SymbolKind;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Type-check an assignment and its target mutability.
    ///
    /// **Documentation:** `docs/pascal/language/basics/variables.md`
    pub(crate) fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        let target_ty = self.check_designator_expr(target);
        let value_ty = self.check_expr_with_expected(value, Some(&target_ty));

        if !target_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "assignment", span);
        }

        let value_is_task_bound = self.expr_is_task_bound(Self::expr_lookup_key(value));
        if target.parts.len() == 1
            && let Some(DesignatorPart::Ident(base, _)) = target.parts.first()
            && let Some(symbol) = self.scopes.lookup_mut(base)
        {
            symbol.task_bound = value_is_task_bound;
        }

        if let Some((symbol, _)) = self.designator_root_symbol(&target.parts)
            && !self.designator_is_mutable_target(target)
        {
            let target_name = Self::resolve_designator_name(target);
            let hint = match symbol.kind {
                SymbolKind::Const => {
                    "A `const` binding cannot be changed; declare it with `var` to allow reassignment."
                }
                SymbolKind::ForVar => "Loop variables are immutable inside the loop body.",
                SymbolKind::Param => {
                    "Parameters are read-only; start the body with a local copy, for example `var LocalValue: integer := Value;`."
                }
                _ => "Declare with `var` to allow reassignment.",
            };

            self.error_with_code(
                SEMA_IMMUTABLE_ASSIGNMENT,
                format!("Cannot assign to `{target_name}`"),
                hint,
                span,
            );
        }
    }
}
