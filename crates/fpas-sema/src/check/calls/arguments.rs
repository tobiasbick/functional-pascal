//! Argument validation for resolved callable signatures.

use super::super::Checker;
use crate::types::{FunctionTy, GenericParamDef, ParamTy, ProcedureTy, Ty, TypeArguments};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashMap;

/// Signature metadata shared by function and procedure call checking.
pub(in crate::check) struct RoutineCallSignature<'a> {
    /// Explicit function purity; procedures and legacy methods are ordinary.
    pub pure: bool,
    /// Resolved name used in diagnostics.
    pub name: &'a str,
    /// Function, procedure, or method diagnostic label.
    pub routine_label: &'a str,
    /// Generic constraint and inference inputs.
    pub type_params: &'a [GenericParamDef],
    /// Positional value/var modes visible to the caller.
    pub params: &'a [ParamTy],
    /// Whether trailing ordinary value arguments are allowed.
    pub variadic: bool,
    /// Arity guidance appropriate to the call syntax.
    pub arity_hint: &'a str,
}

impl Checker {
    /// Check arguments against the resolved callable signature.
    pub(crate) fn check_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        args: &[Expr],
        span: Span,
    ) -> TypeArguments {
        let inferred = self.check_routine_call_args(
            RoutineCallSignature {
                name,
                pure: func_ty.pure,
                routine_label: "Function",
                type_params: &func_ty.type_params,
                params: &func_ty.params,
                variadic: func_ty.variadic,
                arity_hint: "Check the number of arguments.",
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        );
        self.check_pure_instantiation(func_ty, &inferred, span);
        inferred
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
                pure: false,
                routine_label: "Procedure",
                type_params: &proc_ty.type_params,
                params: &proc_ty.params,
                variadic: proc_ty.variadic,
                arity_hint: "Check the number of arguments.",
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        );
    }

    /// Validate modes, roots, type inference, and arity through one resolved signature.
    pub(in crate::check) fn check_routine_call_args(
        &mut self,
        signature: RoutineCallSignature<'_>,
        args: &[&Expr],
        span: Span,
    ) -> TypeArguments {
        let RoutineCallSignature {
            pure,
            name,
            routine_label,
            type_params,
            params,
            variadic,
            arity_hint,
        } = signature;
        self.check_pure_read(name, span);
        if !pure {
            self.reject_impure_operation(&format!("call ordinary routine `{name}`"), span);
        }
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
                arity_hint,
                span,
            );
        }

        let mut arg_types = Vec::with_capacity(args.len());
        let unresolved = type_params
            .iter()
            .map(|parameter| (parameter.identity.clone(), Ty::Error))
            .collect::<HashMap<_, _>>();
        self.inference_depth += 1;
        let mut var_roots = std::collections::BTreeSet::new();
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = if let Some(param) = params.get(index) {
                if param.mutable {
                    let ty = self.check_var_argument(arg);
                    if !ty.is_error()
                        && let Some(root) = self.var_argument_root(arg)
                        && !var_roots.insert(root)
                    {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            "Two var arguments cannot share a storage root",
                            "Pass distinct mutable roots, even when the selected fields or indices differ.",
                            arg.span(),
                        );
                    }
                    ty
                } else {
                    let expected = param.ty.substitute(&unresolved);
                    self.check_expr_with_expected(arg, &expected)
                }
            } else {
                self.check_expr(arg)
            };
            if pure && index >= params.len() && !self.is_pure_data(&arg_ty) {
                self.error_with_code(SEMA_TYPE_MISMATCH,
                    format!("Variadic argument of pure function `{name}` is not resource-free data or a pure callable"),
                    "Pass resource-free data or pure callable values.", arg.span());
            }
            arg_types.push(arg_ty);
        }

        self.inference_depth -= 1;
        let inferred = self.validate_routine_constraints(type_params, params, &arg_types, span);
        for (index, ((param, arg_ty), arg)) in params.iter().zip(&arg_types).zip(args).enumerate() {
            let expected = Self::substitute_type_params(&param.ty, &inferred);
            self.check_nested_pure_signatures(&expected, span);
            let actual = if !param.mutable
                && (arg_ty.has_inference_holes() || !arg_ty.callable_type_parameters().is_empty())
            {
                self.check_expr_with_expected(arg, &expected)
            } else {
                arg_ty.clone()
            };
            if param.mutable
                && !expected.is_error()
                && !actual.is_error()
                && !(expected.compatible_with(&actual) && actual.compatible_with(&expected))
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Var argument {} must have type `{expected}`, found `{actual}`", index + 1),
                    "Var parameters require the exact stored type; implicit value conversions cannot replace caller storage.",
                    arg.span(),
                );
            } else {
                self.check_type_compat(
                    &expected,
                    &actual,
                    &format!("argument {}", index + 1),
                    span,
                );
            }
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
