use super::Checker;
use crate::types::{FunctionTy, GenericParamDef, ParamTy, ProcedureTy, RecordTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Signature metadata shared by function and procedure call checking.
struct RoutineCallSignature<'a> {
    name: &'a str,
    routine_label: &'a str,
    type_params: &'a [GenericParamDef],
    params: &'a [ParamTy],
    variadic: bool,
}

impl Checker {
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
            args,
            span,
        )
    }

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
            args,
            span,
        );
    }

    fn check_routine_call_args(
        &mut self,
        signature: RoutineCallSignature<'_>,
        args: &[Expr],
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
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = if let Some(param) = params.get(index) {
                self.check_expr_with_expected_record_literals(arg, &param.ty)
            } else {
                self.check_expr(arg)
            };
            arg_types.push(arg_ty);
        }

        let inferred = self.validate_routine_constraints(type_params, params, &arg_types, span);
        for (index, ((param, arg_ty), arg)) in params.iter().zip(&arg_types).zip(args).enumerate() {
            let expected = Self::substitute_type_params(&param.ty, &inferred);
            let actual = if matches!(
                (&expected, arg_ty),
                (Ty::Record(expected), Ty::Record(actual))
                    if expected.name != "<anonymous>" && actual.name == "<anonymous>"
            ) {
                self.check_expr_with_expected_record_literals(arg, &expected)
            } else {
                arg_ty.clone()
            };
            self.check_type_compat(&expected, &actual, &format!("argument {}", index + 1), span);
        }
        inferred
    }

    /// Infer type arguments from argument types, enforce consistent reuse, and
    /// validate constraints.
    pub(crate) fn validate_routine_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        params: &[crate::types::ParamTy],
        arg_types: &[Ty],
        span: Span,
    ) -> HashMap<String, Ty> {
        if type_params.is_empty() {
            return HashMap::new();
        }

        let mut inferred = HashMap::new();

        for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
            let mut visited_record_pairs = HashSet::new();
            self.collect_type_param_bindings(
                &param.ty,
                arg_ty,
                &mut inferred,
                &mut visited_record_pairs,
                span,
            );
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

        inferred
    }

    /// Replace routine-level generic parameters with their call-site inferred types.
    pub(crate) fn substitute_type_params(ty: &Ty, inferred: &HashMap<String, Ty>) -> Ty {
        match ty {
            Ty::GenericParam(name, _) => inferred
                .get(&name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            Ty::Array(inner) => Ty::Array(Box::new(Self::substitute_type_params(inner, inferred))),
            Ty::Channel(inner) => {
                Ty::Channel(Box::new(Self::substitute_type_params(inner, inferred)))
            }
            Ty::Result(ok, err) => Ty::Result(
                Box::new(Self::substitute_type_params(ok, inferred)),
                Box::new(Self::substitute_type_params(err, inferred)),
            ),
            Ty::Option(inner) => {
                Ty::Option(Box::new(Self::substitute_type_params(inner, inferred)))
            }
            Ty::Dict(key, value) => Ty::Dict(
                Box::new(Self::substitute_type_params(key, inferred)),
                Box::new(Self::substitute_type_params(value, inferred)),
            ),
            Ty::Task(inner) => Ty::Task(Box::new(Self::substitute_type_params(inner, inferred))),
            Ty::Function(function_ty) => {
                let mut substituted = function_ty.clone();
                for param in &mut substituted.params {
                    param.ty = Self::substitute_type_params(&param.ty, inferred);
                }
                substituted.return_type = Box::new(Self::substitute_type_params(
                    &substituted.return_type,
                    inferred,
                ));
                Ty::Function(substituted)
            }
            Ty::Procedure(procedure_ty) => {
                let mut substituted = procedure_ty.clone();
                for param in &mut substituted.params {
                    param.ty = Self::substitute_type_params(&param.ty, inferred);
                }
                Ty::Procedure(substituted)
            }
            _ => ty.clone(),
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
        visited_record_pairs: &mut HashSet<(*const RecordTy, *const RecordTy)>,
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
                    } else if matches!(
                        (previous, actual_ty),
                        (Ty::Record(previous), Ty::Record(actual))
                            if previous.name == "<anonymous>" && actual.name != "<anonymous>"
                    ) {
                        // A later argument can supply the declaration identity that an earlier
                        // generic record literal did not have context to infer.
                        inferred.insert(key, actual_ty.clone());
                    }
                } else {
                    inferred.insert(key, actual_ty.clone());
                }
            }
            (Ty::Array(declared_inner), Ty::Array(actual_inner)) => {
                self.collect_type_param_bindings(
                    declared_inner,
                    actual_inner,
                    inferred,
                    visited_record_pairs,
                    span,
                );
            }
            (Ty::Channel(declared_inner), Ty::Channel(actual_inner)) => {
                self.collect_type_param_bindings(
                    declared_inner,
                    actual_inner,
                    inferred,
                    visited_record_pairs,
                    span,
                );
            }
            (Ty::Dict(declared_key, declared_value), Ty::Dict(actual_key, actual_value)) => {
                self.collect_type_param_bindings(
                    declared_key,
                    actual_key,
                    inferred,
                    visited_record_pairs,
                    span,
                );
                self.collect_type_param_bindings(
                    declared_value,
                    actual_value,
                    inferred,
                    visited_record_pairs,
                    span,
                );
            }
            (Ty::Option(declared_inner), Ty::Option(actual_inner))
            | (Ty::Task(declared_inner), Ty::Task(actual_inner)) => {
                self.collect_type_param_bindings(
                    declared_inner,
                    actual_inner,
                    inferred,
                    visited_record_pairs,
                    span,
                );
            }
            (Ty::Result(declared_ok, declared_err), Ty::Result(actual_ok, actual_err)) => {
                self.collect_type_param_bindings(
                    declared_ok,
                    actual_ok,
                    inferred,
                    visited_record_pairs,
                    span,
                );
                self.collect_type_param_bindings(
                    declared_err,
                    actual_err,
                    inferred,
                    visited_record_pairs,
                    span,
                );
            }
            (Ty::Record(declared_record), Ty::Record(actual_record)) => {
                // Recursive fields can resolve back to the same descriptor pair.
                // Documentation: docs/pascal/language/types/records.md
                let record_pair = (Arc::as_ptr(declared_record), Arc::as_ptr(actual_record));
                if !visited_record_pairs.insert(record_pair) {
                    return;
                }
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
                            visited_record_pairs,
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
                        visited_record_pairs,
                        span,
                    );
                }
                self.collect_type_param_bindings(
                    &declared_fn.return_type,
                    &actual_fn.return_type,
                    inferred,
                    visited_record_pairs,
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
                        visited_record_pairs,
                        span,
                    );
                }
            }
            _ => {}
        }
    }
}
