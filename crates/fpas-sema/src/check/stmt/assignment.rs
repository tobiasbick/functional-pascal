//! Assignment permissions: `docs/pascal/language/basics/variables.md`.

use super::Checker;
use crate::scope::SymbolKind;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Type-check an assignment to mutable storage.
    ///
    /// **Documentation:** `docs/pascal/language/basics/variables.md`,
    /// `docs/pascal/language/basics/operators.md#string-indexing`
    pub(crate) fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        let checkpoint = self.errors.len();
        let target_ty = self.check_designator_expr(target);
        let target_valid = !target_ty.is_error() && self.errors.len() == checkpoint;
        let value_ty = self.check_expr_with_expected(value, &target_ty);

        if !target_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "assignment", span);
        }

        if target_valid && self.reject_string_index_assignment(target) {
            return;
        }

        let value_is_task_bound = self.expr_is_task_bound(Self::expr_lookup_key(value));
        if target.parts.len() == 1
            && let Some(DesignatorPart::Ident(base, _)) = target.parts.first()
            && let Some(symbol) = self.scopes.lookup_mut(base)
        {
            symbol.task_bound = value_is_task_bound;
        }

        let root = self
            .designator_root_symbol(&target.parts)
            .map(|(symbol, _)| (symbol.mutable, symbol.kind));
        if target_valid
            && let Some((mutable, kind)) = root
            && !(mutable && matches!(kind, SymbolKind::Var | SymbolKind::VarParam))
        {
            let target_name = self.resolve_designator_name(target);
            let hint = match kind {
                SymbolKind::Const => {
                    "Declare the binding with `var` to allow reassignment; `const` bindings are immutable."
                }
                SymbolKind::ForVar => "Loop variables are immutable inside the loop body.",
                SymbolKind::Param => {
                    "Use a local mutable copy, or declare a `var` parameter for explicit caller mutation."
                }
                _ => {
                    "Declare the binding with `var` to allow reassignment; `const` bindings are immutable."
                }
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
