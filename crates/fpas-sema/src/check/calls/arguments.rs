//! Argument validation for resolved callable signatures.

use super::super::Checker;
use crate::types::{FunctionTy, GenericParamDef, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashMap;

/// Signature metadata shared by function and procedure call checking.
struct RoutineCallSignature<'a> {
    name: &'a str,
    routine_label: &'a str,
    type_params: &'a [GenericParamDef],
    params: &'a [ParamTy],
    variadic: bool,
}

impl Checker {
    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Function",
                type_params: &func_ty.type_params,
                params: &func_ty.params,
                variadic: func_ty.variadic,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        )
    }

    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_procedure_call_args(
        &mut self,
        name: &str,
        proc_ty: &ProcedureTy,
        args: &[Expr],
        span: Span,
    ) {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Procedure",
                type_params: &proc_ty.type_params,
                params: &proc_ty.params,
                variadic: proc_ty.variadic,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        );
    }

    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_fluent_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        args: &[&Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Function",
                type_params: &func_ty.type_params,
                params: &func_ty.params,
                variadic: func_ty.variadic,
            },
            args,
            span,
        )
    }

    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_fluent_procedure_call_args(
        &mut self,
        name: &str,
        proc_ty: &ProcedureTy,
        args: &[&Expr],
        span: Span,
    ) {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Procedure",
                type_params: &proc_ty.type_params,
                params: &proc_ty.params,
                variadic: proc_ty.variadic,
            },
            args,
            span,
        );
    }

    fn check_routine_call_args(
        &mut self,
        signature: RoutineCallSignature<'_>,
        args: &[&Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        let RoutineCallSignature {
            name,
            routine_label,
            type_params,
            params,
            variadic,
        } = signature;
        if variadic && args.len() < params.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "{routine_label} `{name}` expects at least {} arguments, got {}",
                    params.len(),
                    args.len()
                ),
                "Pass all required arguments before any variadic arguments.",
                span,
            );
        } else if !variadic && params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "{routine_label} `{name}` expects {} arguments, got {}",
                    params.len(),
                    args.len()
                ),
                "Check the number of arguments.",
                span,
            );
        }

        let mut arg_types = Vec::with_capacity(args.len());
        let unresolved = type_params
            .iter()
            .map(|parameter| (parameter.name.to_ascii_lowercase(), Ty::Error))
            .collect::<HashMap<_, _>>();
        self.inference_depth += 1;
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = if let Some(param) = params.get(index) {
                let expected = param.ty.substitute(&unresolved);
                self.check_expr_with_expected(arg, &expected)
            } else {
                self.check_expr(arg)
            };
            arg_types.push(arg_ty);
        }

        self.inference_depth -= 1;
        let inferred = self.validate_routine_constraints(type_params, params, &arg_types, span);
        for (index, ((param, arg_ty), arg)) in params.iter().zip(&arg_types).zip(args).enumerate() {
            let expected = Self::substitute_type_params(&param.ty, &inferred);
            let actual = if arg_ty.has_inference_holes() {
                self.check_expr_with_expected(arg, &expected)
            } else {
                arg_ty.clone()
            };
            self.check_type_compat(&expected, &actual, &format!("argument {}", index + 1), span);
        }
        inferred
    }

    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_args_only(&mut self, args: &[Expr]) {
        for arg in args {
            self.check_expr(arg);
        }
    }
}
