//! Shared joint inference for record fields and enum payloads.
//! See `docs/pascal/language/types/generics.md`.

use crate::{
    check::Checker,
    types::{GenericParamDef, Ty, has_inference_holes, needs_expected_type},
};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::{HashMap, HashSet};

/// Supplied value, declaration type, and optional contextual field type.
pub(super) struct ConstructorField<'a> {
    /// Explicitly supplied field expression.
    pub value: &'a Expr,
    /// Field type in the generic declaration.
    pub declared: &'a Ty,
    /// Concrete field type supplied by the expected application.
    pub expected: Option<&'a Ty>,
}

impl Checker {
    /// Infer all supplied fields together, then complete nested constructors.
    pub(super) fn infer_constructor_arguments(
        &mut self,
        name: &str,
        parameters: &[GenericParamDef],
        fixed_arguments: &[Ty],
        fields: &[ConstructorField<'_>],
        span: Span,
    ) -> Vec<Ty> {
        let mut inferred = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(fixed_arguments) {
            self.collect_type_param_bindings(
                &Ty::GenericParam(parameter.name.clone(), parameter.constraint),
                argument,
                &mut inferred,
                &mut HashSet::new(),
                span,
            );
        }
        self.deferred_constructor_inference += 1;
        for field in fields {
            let actual = self.check_expr_with_expected(field.value, field.expected);
            self.collect_type_param_bindings(
                field.declared,
                &actual,
                &mut inferred,
                &mut HashSet::new(),
                field.value.span(),
            );
            self.prechecked_receivers
                .insert(Self::expr_lookup_key(field.value), actual);
        }
        self.deferred_constructor_inference -= 1;
        let arguments: Vec<_> = parameters
            .iter()
            .map(|parameter| {
                inferred
                    .get(&parameter.name.to_ascii_lowercase())
                    .cloned()
                    .unwrap_or(Ty::Error)
            })
            .collect();
        let missing: Vec<_> = parameters
            .iter()
            .zip(&arguments)
            .filter(|(_, ty)| has_inference_holes(ty))
            .map(|(parameter, _)| parameter.display_name())
            .collect();
        if !missing.is_empty() && self.deferred_constructor_inference == 0 {
            self.error_with_code(SEMA_TYPE_MISMATCH,
                format!("Cannot infer type parameter(s) {} for `{name}`", missing.join(", ")),
                "Add a type annotation, such as `const Value: Lookup of string := Lookup.Missing;`. Record defaults do not infer type arguments.", span);
        }
        self.validate_constraints(parameters, &arguments, span);
        for field in fields {
            let key = Self::expr_lookup_key(field.value);
            let Some(actual) = self.prechecked_receivers.get(&key) else {
                continue;
            };
            if actual.is_error() || !needs_expected_type(actual) {
                continue;
            }
            let expected = Self::substitute_type_params(field.declared, &inferred);
            if has_inference_holes(&expected) {
                continue;
            }
            self.prechecked_receivers.remove(&key);
            let actual = self.check_expr_with_expected(field.value, Some(&expected));
            self.prechecked_receivers.insert(key, actual);
        }
        arguments
    }
}
