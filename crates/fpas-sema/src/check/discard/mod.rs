//! Ordinary value discard and required consumption of standalone function results.
//!
//! **Documentation:** `docs/pascal/language/functions/discard.md`

mod unused_results;

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_DISCARD_REQUIRES_VALUE;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Evaluates an ordinary value once; task owners remain independent of its aliases.
    pub(crate) fn check_discard(&mut self, expr: &Expr, span: Span) {
        let mut operand = expr;
        while let Expr::Paren(inner, _) = operand {
            operand = inner;
        }
        if let Expr::Call {
            designator,
            args,
            span: call_span,
        } = operand
        {
            let name = Self::resolve_designator_name(designator);
            if self
                .scopes
                .lookup(&name)
                .is_some_and(|symbol| matches!(symbol.ty, Ty::Procedure(_)))
            {
                self.check_call_stmt(designator, args, *call_span);
                self.error_with_code(
                    SEMA_DISCARD_REQUIRES_VALUE,
                    "`discard` requires a value; a procedure call has no result",
                    "Call the procedure directly without `discard`.",
                    span,
                );
                return;
            }
        }
        let ty = self.check_expr(expr);
        if ty.is_error() {
            return;
        }
        if matches!(ty, Ty::Unit) {
            self.error_with_code(
                SEMA_DISCARD_REQUIRES_VALUE,
                "`discard` requires a value; a procedure call has no result",
                "Call the procedure directly without `discard`.",
                span,
            );
            return;
        }
    }
}
