//! Instantiate generic callable values from an explicit or argument signature context.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::Checker;
use crate::types::{Ty, TypeArguments};
use fpas_lexer::Span;

impl Checker {
    pub(super) fn instantiate_callable_with_expected(
        &mut self,
        actual: &Ty,
        expected: &Ty,
        span: Span,
    ) -> Option<Ty> {
        if !matches!(
            (actual, expected),
            (Ty::Function(_), Ty::Function(_)) | (Ty::Procedure(_), Ty::Procedure(_))
        ) {
            return None;
        }
        let parameters = actual.callable_type_parameters();
        if parameters.is_empty() {
            return None;
        }
        let mut inferred = TypeArguments::new();
        self.collect_type_param_bindings(actual, expected, &mut inferred, parameters, span);
        let arguments = parameters
            .iter()
            .map(|parameter| inferred.get(&parameter.identity).cloned())
            .collect::<Option<Vec<_>>>()?;
        if arguments.iter().any(Ty::has_inference_holes) {
            return None;
        }
        self.validate_constraints(parameters, &arguments, span);
        if let Ty::Function(function) = actual {
            self.check_pure_instantiation(function, &inferred, span);
        }
        let mut instantiated = actual.substitute(&inferred);
        match &mut instantiated {
            Ty::Function(function) => function.type_params.clear(),
            Ty::Procedure(procedure) => procedure.type_params.clear(),
            _ => return None,
        }
        Some(instantiated)
    }
}
