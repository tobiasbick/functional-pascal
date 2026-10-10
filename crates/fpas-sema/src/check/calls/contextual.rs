//! Expected types for record constructors and generic callable values.
//! See `docs/pascal/language/types/generics.md`.

use crate::{
    check::Checker,
    types::{GenericParamDef, ParamTy, Ty, has_inference_holes, needs_expected_type},
};
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};

/// Leave unbound callee parameters unknown until supplied arguments determine them.
pub(super) fn contextual_parameters(
    type_params: &[GenericParamDef],
    params: &[ParamTy],
    inferred: &HashMap<String, Ty>,
) -> Vec<ParamTy> {
    let mut bindings: HashMap<_, _> = type_params
        .iter()
        .map(|parameter| (parameter.name.to_ascii_lowercase(), Ty::Error))
        .collect();
    bindings.extend(inferred.iter().map(|(name, ty)| (name.clone(), ty.clone())));
    params
        .iter()
        .map(|param| ParamTy {
            ty: Checker::substitute_type_params(&param.ty, &bindings),
            ..param.clone()
        })
        .collect()
}

impl Checker {
    /// Complete deferred constructors after every routine argument has contributed.
    pub(super) fn complete_constructor_arguments(
        &mut self,
        params: &[ParamTy],
        args: &[&Expr],
        arg_types: &mut [Ty],
    ) {
        for ((param, argument), actual) in params.iter().zip(args).zip(arg_types) {
            if !actual.is_error() && needs_expected_type(actual) {
                *actual = self.check_expr_with_expected(argument, Some(&param.ty));
            }
        }
    }

    /// Instantiate a generic routine value using the expected callable signature.
    pub(in crate::check) fn specialize_expected_callable(
        &mut self,
        actual: Ty,
        expected: &Ty,
        span: Span,
    ) -> Ty {
        let type_params = match (&actual, expected) {
            (Ty::Function(function), Ty::Function(_)) => &function.type_params,
            (Ty::Procedure(procedure), Ty::Procedure(_)) => &procedure.type_params,
            _ => return actual,
        };
        if type_params.is_empty() {
            return actual;
        }
        let mut inferred = HashMap::new();
        self.collect_type_param_bindings(
            &actual,
            expected,
            &mut inferred,
            &mut HashSet::new(),
            span,
        );
        let Some(arguments) = type_params
            .iter()
            .map(|parameter| inferred.get(&parameter.name.to_ascii_lowercase()).cloned())
            .collect::<Option<Vec<_>>>()
        else {
            return actual;
        };
        if arguments.iter().any(has_inference_holes) {
            return actual;
        }
        self.validate_constraints(type_params, &arguments, span);
        match Self::substitute_type_params(&actual, &inferred) {
            Ty::Function(mut function) => {
                function.type_params.clear();
                Ty::Function(function)
            }
            Ty::Procedure(mut procedure) => {
                procedure.type_params.clear();
                Ty::Procedure(procedure)
            }
            other => other,
        }
    }
}
