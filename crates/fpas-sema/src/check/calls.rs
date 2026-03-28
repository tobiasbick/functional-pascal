use super::Checker;
use crate::types::{FunctionTy, ProcedureTy};
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;

impl Checker {
    pub(crate) fn check_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        args: &[Expr],
        span: Span,
    ) {
        if func_ty.params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Function `{name}` expects {} arguments, got {}",
                    func_ty.params.len(),
                    args.len()
                ),
                "Check the number of arguments.",
                span,
            );
        }

        for (index, arg) in args.iter().enumerate() {
            let arg_ty = self.check_expr(arg);
            if let Some(param) = func_ty.params.get(index) {
                self.check_type_compat(
                    &param.ty,
                    &arg_ty,
                    &format!("argument {}", index + 1),
                    span,
                );
            }
        }
    }

    pub(crate) fn check_procedure_call_args(
        &mut self,
        name: &str,
        proc_ty: &ProcedureTy,
        args: &[Expr],
        span: Span,
    ) {
        if !proc_ty.variadic && proc_ty.params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Procedure `{name}` expects {} arguments, got {}",
                    proc_ty.params.len(),
                    args.len()
                ),
                "Check the number of arguments.",
                span,
            );
        }

        for (index, arg) in args.iter().enumerate() {
            let arg_ty = self.check_expr(arg);
            if let Some(param) = proc_ty.params.get(index) {
                self.check_type_compat(
                    &param.ty,
                    &arg_ty,
                    &format!("argument {}", index + 1),
                    span,
                );
            }
        }
    }

    pub(crate) fn check_args_only(&mut self, args: &[Expr]) {
        for arg in args {
            self.check_expr(arg);
        }
    }
}
