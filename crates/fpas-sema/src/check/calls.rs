use super::Checker;
use crate::types::{FunctionTy, GenericParamDef, ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashMap;

impl Checker {
    pub(crate) fn check_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        args: &[Expr],
        span: Span,
    ) {
        if !func_ty.variadic && func_ty.params.len() != args.len() {
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

    /// Infer type arguments from argument types, enforce consistent reuse, and
    /// validate constraints.
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

        let mut inferred = HashMap::new();

        for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
            self.collect_type_param_bindings(&param.ty, arg_ty, &mut inferred, span);
        }

        // Build a Vec<GenericParamDef> + Vec<Ty> for only the params we inferred.
        let mut check_params = Vec::new();
        let mut check_args = Vec::new();
        for tp in type_params {
            if let Some(arg) = inferred.get(&tp.name.to_ascii_lowercase()) {
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

    fn collect_type_param_bindings(
        &mut self,
        declared: &Ty,
        actual: &Ty,
        inferred: &mut HashMap<String, Ty>,
        span: Span,
    ) {
        let declared_visible = self.resolve_visible_type(declared);
        let actual_visible = self.resolve_visible_type(actual);

        match (&declared_visible, &actual_visible) {
            (Ty::GenericParam(name, _), actual_ty)
                if !actual_ty.is_error() && !matches!(actual_ty, Ty::GenericParam(..)) =>
            {
                let key = name.to_ascii_lowercase();
                if let Some(previous) = inferred.get(&key) {
                    if !previous.compatible_with(actual_ty) || !actual_ty.compatible_with(previous)
                    {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            format!(
                                "Type parameter `{name}` was inferred as `{previous}`, but was also used with `{actual_ty}`"
                            ),
                            format!(
                                "Use the same concrete type for every argument bound to `{name}`."
                            ),
                            span,
                        );
                    }
                } else {
                    inferred.insert(key, actual_ty.clone());
                }
            }
            (Ty::Array(declared_inner), Ty::Array(actual_inner)) => {
                self.collect_type_param_bindings(declared_inner, actual_inner, inferred, span);
            }
            (Ty::Dict(declared_key, declared_value), Ty::Dict(actual_key, actual_value)) => {
                self.collect_type_param_bindings(declared_key, actual_key, inferred, span);
                self.collect_type_param_bindings(declared_value, actual_value, inferred, span);
            }
            (Ty::Option(declared_inner), Ty::Option(actual_inner))
            | (Ty::Task(declared_inner), Ty::Task(actual_inner)) => {
                self.collect_type_param_bindings(declared_inner, actual_inner, inferred, span);
            }
            (Ty::Result(declared_ok, declared_err), Ty::Result(actual_ok, actual_err)) => {
                self.collect_type_param_bindings(declared_ok, actual_ok, inferred, span);
                self.collect_type_param_bindings(declared_err, actual_err, inferred, span);
            }
            (Ty::Record(declared_record), Ty::Record(actual_record)) => {
                for (field_name, declared_field_ty) in &declared_record.fields {
                    if let Some((_, actual_field_ty)) = actual_record
                        .fields
                        .iter()
                        .find(|(actual_name, _)| actual_name.eq_ignore_ascii_case(field_name))
                    {
                        self.collect_type_param_bindings(
                            declared_field_ty,
                            actual_field_ty,
                            inferred,
                            span,
                        );
                    }
                }
            }
            (Ty::Function(declared_fn), Ty::Function(actual_fn)) => {
                for (declared_param, actual_param) in
                    declared_fn.params.iter().zip(actual_fn.params.iter())
                {
                    self.collect_type_param_bindings(
                        &declared_param.ty,
                        &actual_param.ty,
                        inferred,
                        span,
                    );
                }
                self.collect_type_param_bindings(
                    &declared_fn.return_type,
                    &actual_fn.return_type,
                    inferred,
                    span,
                );
            }
            (Ty::Procedure(declared_proc), Ty::Procedure(actual_proc)) => {
                for (declared_param, actual_param) in
                    declared_proc.params.iter().zip(actual_proc.params.iter())
                {
                    self.collect_type_param_bindings(
                        &declared_param.ty,
                        &actual_param.ty,
                        inferred,
                        span,
                    );
                }
            }
            _ => {}
        }
    }
}
