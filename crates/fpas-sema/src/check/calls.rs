use super::Checker;
use crate::types::{FunctionTy, GenericParamDef, ProcedureTy, Ty};
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

        let mut arg_types = Vec::with_capacity(args.len());
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
            arg_types.push(arg_ty);
        }

        self.validate_routine_constraints(&func_ty.type_params, &func_ty.params, &arg_types, span);
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

        let mut arg_types = Vec::with_capacity(args.len());
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
            arg_types.push(arg_ty);
        }

        self.validate_routine_constraints(&proc_ty.type_params, &proc_ty.params, &arg_types, span);
    }

    /// Infer type arguments from argument types and validate constraints.
    ///
    /// For each type parameter, find the first parameter whose declared type is
    /// `GenericParam(name, _)` and use the corresponding argument type as the
    /// inferred concrete type. Then validate each inferred type against its
    /// constraint (if any).
    pub(crate) fn validate_routine_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        params: &[crate::types::ParamTy],
        arg_types: &[Ty],
        span: Span,
    ) {
        if type_params.is_empty() {
            return;
        }

        let mut inferred: Vec<Option<Ty>> = vec![None; type_params.len()];

        for (i, tp) in type_params.iter().enumerate() {
            for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
                if let Ty::GenericParam(ref name, _) = param.ty {
                    if name.eq_ignore_ascii_case(&tp.name)
                        && !arg_ty.is_error()
                        && !matches!(arg_ty, Ty::GenericParam(..))
                    {
                        inferred[i] = Some(arg_ty.clone());
                        break;
                    }
                }
            }
        }

        // Build a Vec<GenericParamDef> + Vec<Ty> for only the params we inferred.
        let mut check_params = Vec::new();
        let mut check_args = Vec::new();
        for (i, tp) in type_params.iter().enumerate() {
            if let Some(ref arg) = inferred[i] {
                check_params.push(tp.clone());
                check_args.push(arg.clone());
            }
        }

        if !check_params.is_empty() {
            self.validate_constraints(&check_params, &check_args, span);
        }
    }

    pub(crate) fn check_args_only(&mut self, args: &[Expr]) {
        for arg in args {
            self.check_expr(arg);
        }
    }
}
