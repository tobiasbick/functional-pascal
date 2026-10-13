//! Boolean expression checking for branch and loop conditions.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`

use super::super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_BOOLEAN_CONDITION;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Checks an ordinary Boolean expression used by a branch or loop.
    pub(in crate::check) fn check_branch_condition(
        &mut self,
        condition: &Expr,
        keyword: &str,
        span: Span,
    ) {
        let condition_ty = self.check_expr(condition);
        self.require_boolean_condition(&condition_ty, keyword, span);
    }

    fn require_boolean_condition(&mut self, ty: &Ty, keyword: &str, span: Span) {
        if matches!(ty, Ty::GenericParam(..)) {
            self.check_type_compat(&Ty::Boolean, ty, &format!("{keyword} condition"), span);
        } else if !Ty::Boolean.assignment_compatible_with(ty) {
            let (message, hint) = if keyword == "while" {
                (
                    "While condition must be a boolean expression",
                    "while <boolean> do ...",
                )
            } else {
                (
                    "Condition must be a boolean expression",
                    "if <boolean> then ...",
                )
            };
            self.error_with_code(SEMA_NON_BOOLEAN_CONDITION, message, hint, span);
        }
    }
}
