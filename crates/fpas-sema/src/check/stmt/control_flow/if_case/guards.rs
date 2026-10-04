//! Boolean guard checking shared by statement and expression cases.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/guards.md`.

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_BOOLEAN_CONDITION;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Check a guard after the arm's immutable pattern bindings enter scope.
    pub(in crate::check) fn check_guard(&mut self, guard: &Option<Expr>, span: Span) {
        if let Some(guard_expr) = guard {
            let guard_ty = self.check_expr(guard_expr);
            if matches!(guard_ty, Ty::GenericParam(..)) {
                self.check_type_compat(&Ty::Boolean, &guard_ty, "case guard", span);
            } else if !Ty::Boolean.assignment_compatible_with(&guard_ty) {
                self.error_with_code(
                    SEMA_NON_BOOLEAN_CONDITION,
                    "Guard clause must be a boolean expression",
                    "when Pattern if <boolean>: ...",
                    span,
                );
            }
        }
    }
}
