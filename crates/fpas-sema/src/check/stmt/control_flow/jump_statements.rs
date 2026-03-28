use super::super::super::Checker;
use fpas_diagnostics::codes::{SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    pub(in super::super) fn check_return_stmt(&mut self, expr: Option<&Expr>, span: Span) {
        let function_ctx = self.scopes.function_ctx.clone();
        match function_ctx {
            Some(ref function_ctx) => match (&function_ctx.return_type, expr) {
                (Some(expected), Some(expr)) => {
                    let actual = self.check_expr(expr);
                    self.check_type_compat(expected, &actual, "return value", span);
                }
                (Some(expected), None) => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Function `{}` must return a value of type `{:?}`",
                            function_ctx.name, expected
                        ),
                        "Add a return expression: return <expr>",
                        span,
                    );
                }
                (None, Some(expr)) => {
                    self.check_expr(expr);
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Procedure `{}` cannot return a value", function_ctx.name),
                        "Use bare `return` to exit a procedure.",
                        span,
                    );
                }
                (None, None) => {}
            },
            None => {
                if let Some(expr) = expr {
                    self.check_expr(expr);
                }
            }
        }
    }

    pub(in super::super) fn check_break_stmt(&mut self, span: Span) {
        if self.scopes.loop_depth == 0 {
            self.error_with_code(
                SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT,
                "`break` can only be used inside a loop",
                "Place break inside for, while, or repeat.",
                span,
            );
        }
    }

    pub(in super::super) fn check_continue_stmt(&mut self, span: Span) {
        if self.scopes.loop_depth == 0 {
            self.error_with_code(
                SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT,
                "`continue` can only be used inside a loop",
                "Place continue inside for, while, or repeat.",
                span,
            );
        }
    }
}
