//! Calls of ordinary callable values, without an implicit receiver.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    /// Checks a call through a callable value with no implicit receiver.
    pub(in crate::check) fn check_value_call(
        &mut self,
        call_key: usize,
        name: &str,
        member_ty: &Ty,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Ty {
        let result = match member_ty {
            Ty::Function(signature) => {
                let inferred = self.check_function_call_args(name, signature, args, span);
                Self::substitute_type_params(&signature.return_type, &inferred)
            }
            Ty::Procedure(signature) => {
                self.check_procedure_call_args(name, signature, args, span);
                Ty::Unit
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Callable target `{name}` is not callable"),
                    "Use a value with a function or procedure type.",
                    span,
                );
                self.check_args_only(args);
                return Ty::Error;
            }
        };
        if result == Ty::Unit && !allow_procedure_result {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Callable target `{name}` does not return a value"),
                "Use the call as the final operation of a statement.",
                span,
            );
            return Ty::Error;
        }
        self.value_calls.insert(
            call_key,
            crate::check::ValueCallTarget {
                callable_ty: member_ty.clone(),
                result_ty: result.clone(),
                call_span: span,
            },
        );
        result
    }
}
