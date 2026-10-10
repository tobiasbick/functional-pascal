//! Argument checking for routine, method, and function-value calls.
//!
//! **Documentation:** `docs/pascal/language/functions/parameters.md`

mod contextual;
mod inference;
mod named;

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::{FunctionTy, GenericParamDef, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashMap;

/// Call target kinds that differ in named-argument support.
///
/// **Documentation:** `docs/pascal/language/functions/parameters.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallTarget {
    /// Declared function, procedure, or record method, including standard-library routines.
    Routine,
    /// Closure, callable binding, parameter, or callable record member.
    FunctionValue,
    /// Enum variant constructor; its fields act as parameters.
    EnumVariant,
}

impl CallTarget {
    /// Classifies a resolved callable symbol.
    pub(crate) fn for_symbol(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Function | SymbolKind::Procedure => Self::Routine,
            _ => Self::FunctionValue,
        }
    }
}

/// Signature metadata shared by function, procedure, and method call checking.
struct RoutineCallSignature<'a> {
    name: &'a str,
    routine_label: &'a str,
    arity_hint: &'a str,
    target: CallTarget,
    type_params: &'a [GenericParamDef],
    params: &'a [ParamTy],
    variadic: bool,
}

impl Checker {
    pub(crate) fn check_function_call_args(
        &mut self,
        name: &str,
        func_ty: &FunctionTy,
        target: CallTarget,
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Function",
                arity_hint: "Check the number of arguments.",
                target,
                type_params: &func_ty.type_params,
                params: &func_ty.params,
                variadic: func_ty.variadic,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        )
    }

    pub(crate) fn check_procedure_call_args(
        &mut self,
        name: &str,
        proc_ty: &ProcedureTy,
        target: CallTarget,
        args: &[Expr],
        span: Span,
    ) {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Procedure",
                arity_hint: "Check the number of arguments.",
                target,
                type_params: &proc_ty.type_params,
                params: &proc_ty.params,
                variadic: proc_ty.variadic,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        );
    }

    /// Checks pre-mapped native argument references using ordinary parameter rules.
    pub(crate) fn check_function_call_refs(
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
                arity_hint: "Check the number of arguments.",
                target: CallTarget::Routine,
                type_params: &func_ty.type_params,
                params: &func_ty.params,
                variadic: func_ty.variadic,
            },
            args,
            span,
        )
    }

    /// Checks instance-method arguments against the parameters after `Self`.
    pub(crate) fn check_method_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Method",
                arity_hint: "Check the number of arguments (Self is implicit).",
                target: CallTarget::Routine,
                type_params,
                params: visible_params,
                variadic: false,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        )
    }

    /// Checks static record routine arguments.
    pub(crate) fn check_static_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_routine_call_args(
            RoutineCallSignature {
                name,
                routine_label: "Method",
                arity_hint: "Check the number of arguments.",
                target: CallTarget::Routine,
                type_params,
                params,
                variadic: false,
            },
            &args.iter().collect::<Vec<_>>(),
            span,
        )
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
            arity_hint,
            target,
            type_params,
            params,
            variadic,
        } = signature;
        let named = args.iter().any(|arg| arg.argument_name().is_some());
        let Some(args) = self.order_call_arguments(name, target, params, variadic, args) else {
            return HashMap::new();
        };
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

        let mut argument_indices = (0..args.len()).collect::<Vec<_>>();
        if named {
            // Mapping changes parameter order; checking still follows the source order.
            argument_indices.sort_unstable_by_key(|&index| args[index].span().offset);
        }
        let mut arg_types = vec![Ty::Error; args.len()];
        let mut var_roots = Vec::new();
        let contextual = contextual::contextual_parameters(type_params, params, &HashMap::new());
        if !type_params.is_empty() {
            self.deferred_constructor_inference += 1;
        }
        for index in argument_indices {
            let arg = args[index];
            let arg_ty = if let Some(param) = contextual.get(index) {
                let (ty, root) = self.check_argument_for_param(name, param, arg, named);
                var_roots.extend(root);
                ty
            } else {
                let ty = self.check_expr(arg);
                self.reject_implicit_distinct_unwrap(name, &ty, arg.span());
                ty
            };
            arg_types[index] = arg_ty;
        }
        if !type_params.is_empty() {
            self.deferred_constructor_inference -= 1;
        }
        self.reject_var_argument_aliases(&var_roots, span);

        let inferred = self.validate_routine_constraints(type_params, params, &arg_types, span);
        if !type_params.is_empty() {
            let contextual = contextual::contextual_parameters(type_params, params, &inferred);
            self.complete_constructor_arguments(&contextual, &args, &mut arg_types);
        }
        for (index, (param, actual)) in params.iter().zip(&arg_types).enumerate() {
            let expected = Self::substitute_type_params(&param.ty, &inferred);
            let role = if named {
                format!("argument `{}`", param.name)
            } else {
                format!("argument {}", index + 1)
            };
            self.check_type_compat(&expected, actual, &role, span);
        }
        inferred
    }

    /// Checks argument values after the call itself was rejected.
    pub(crate) fn check_args_only(&mut self, args: &[Expr]) {
        for arg in args {
            self.check_expr(arg.argument_value());
        }
    }
}
