//! Expected call results complete missing generic arguments after supplied values.
//! Documentation: docs/pascal/language/functions/generic-routines.md.

use crate::{
    check::Checker,
    types::{GenericParamDef, Ty, needs_expected_type},
};
use fpas_lexer::Span;
use std::collections::{HashMap, HashSet};

impl Checker {
    /// Fills only unresolved argument types from an expected result.
    pub(super) fn complete_result_arguments(
        &mut self,
        parameters: &[GenericParamDef],
        result: Option<(&Ty, Option<&Ty>)>,
        inferred: &mut HashMap<String, Ty>,
        span: Span,
    ) {
        let Some((declared, Some(expected))) = result else {
            return;
        };
        let mut context = HashMap::new();
        self.collect_type_param_bindings(
            declared,
            expected,
            &mut context,
            &mut HashSet::new(),
            span,
        );
        for parameter in parameters {
            let key = parameter.name.to_ascii_lowercase();
            let Some(candidate) = context.get(&key) else {
                continue;
            };
            if needs_expected_type(candidate) {
                continue;
            }
            match inferred.get(&key) {
                Some(previous) if !needs_expected_type(previous) => {}
                Some(_) => {
                    // Rechecking supplied expressions under this complete context specializes
                    // nested callables and checks every already-known fragment for conflicts.
                    inferred.insert(key, candidate.clone());
                }
                None => {
                    inferred.insert(key, candidate.clone());
                }
            }
        }
    }

    /// Checks constraints and rejects generic parameters still lacking evidence.
    pub(super) fn check_inferred_arguments(
        &mut self,
        name: &str,
        parameters: &[GenericParamDef],
        result: Option<&Ty>,
        inferred: &HashMap<String, Ty>,
        span: Span,
    ) {
        let mut missing = Vec::new();
        let mut known = Vec::new();
        let mut arguments = Vec::new();
        for parameter in parameters {
            match inferred.get(&parameter.name.to_ascii_lowercase()) {
                Some(ty) if !needs_expected_type(ty) => {
                    known.push(parameter.clone());
                    arguments.push(ty.clone());
                }
                _ => missing.push(parameter.display_name()),
            }
        }
        self.validate_constraints(&known, &arguments, span);
        let mut result_parameters = HashMap::new();
        if self.deferred_constructor_inference > 0
            && let Some(result) = result
        {
            self.collect_type_param_bindings(
                result,
                result,
                &mut result_parameters,
                &mut HashSet::new(),
                span,
            );
        }
        let cannot_defer = parameters.iter().any(|parameter| {
            inferred
                .get(&parameter.name.to_ascii_lowercase())
                .is_none_or(needs_expected_type)
                && !result_parameters.contains_key(&parameter.name.to_ascii_lowercase())
        });
        self.complete_or_defer_type_arguments(name, missing.into_iter().map(str::to_owned).collect(),
            "Supply typed arguments or a result annotation, for example `const Value: Option of integer := Empty();`. A typed callable can also supply context. Remove unused type parameters that have no evidence.",
            span, !cannot_defer);
    }
}
