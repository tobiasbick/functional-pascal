//! Explicit result consumption and task-handle protection.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Require an explicit consumer for a function call used as a statement.
    pub(in crate::check) fn require_consumed_call_result(&mut self, ty: &Ty, span: Span) {
        if !ty.is_error() && *ty != Ty::Unit {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Function result must be consumed",
                "Store or use the result, or write `discard Function(...)`.",
                span,
            );
        }
    }

    /// Check a discarded value, including task handles nested in aggregate types.
    pub(super) fn check_discard_stmt(&mut self, expr: &Expr, span: Span) {
        let ty = self.check_expr(expr);
        if ty.contains_task_with(|ty| self.resolve_visible_type(ty)) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Cannot discard a task handle or a value containing task handles",
                "Retain the task handle and wait for its result explicitly.",
                span,
            );
        } else if ty == Ty::Unit {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Discard requires a value",
                "Call a procedure directly as a statement; it produces no value.",
                span,
            );
        }
    }
}
