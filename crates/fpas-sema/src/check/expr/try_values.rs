//! Result and Option propagation and enclosing return-type compatibility.
//!
//! **Documentation:** `docs/pascal/language/error-handling/try.md`.

use super::Checker;
use crate::types::Ty;
use fpas_parser::Expr;

impl Checker {
    pub(super) fn check_try_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = self.check_expr(inner);
        match &inner_ty {
            Ty::Result(ok, _) => {
                self.check_try_context(&inner_ty, span);
                *ok.clone()
            }
            Ty::Option(inner) => {
                self.check_try_context(&inner_ty, span);
                *inner.clone()
            }
            Ty::Error => Ty::Error,
            _ => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("try requires Result or Option, found `{inner_ty}`"),
                    "Use try only on Result or Option values.".to_string(),
                    span,
                );
                Ty::Error
            }
        }
    }

    fn check_try_context(&mut self, inner_ty: &Ty, span: fpas_lexer::Span) {
        let Some(function_ctx) = self.scopes.function_ctx.clone() else {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                "`try` can only be used inside a function that returns Result or Option",
                "Wrap the expression in a function that returns `Result of (T, E)` or `Option of (T)`.",
                span,
            );
            return;
        };

        let Some(return_ty) = function_ctx.return_type else {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                format!(
                    "Procedure `{}` cannot use `try` because it does not return a value",
                    function_ctx.name
                ),
                "Use `try` inside a function that returns `Result of (T, E)` or `Option of (T)`.",
                span,
            );
            return;
        };

        if return_ty.is_error() {
            return;
        }

        match (inner_ty, &return_ty) {
            (Ty::Result(_, inner_err), Ty::Result(_, outer_err)) => {
                if !outer_err.compatible_with(inner_err) {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
                            function_ctx.name
                        ),
                        "Make the enclosing function return `Result of (ValueType, ErrorType)`.",
                        span,
                    );
                }
            }
            (Ty::Option(_), Ty::Option(_)) => {}
            (Ty::Result(_, _), _) => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!(
                        "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
                        function_ctx.name
                    ),
                    "Use `try` on `Result` only inside a function that returns `Result of (T, E)` with a compatible error type.",
                    span,
                );
            }
            (Ty::Option(_), _) => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!(
                        "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
                        function_ctx.name
                    ),
                    "Use `try` on `Option` only inside a function that returns `Option of (T)`.",
                    span,
                );
            }
            _ => {}
        }
    }
}
