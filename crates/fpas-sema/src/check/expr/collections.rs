//! Joint element evidence and contextual completion for collection literals.
//! Documentation: docs/pascal/language/basics/local-variables.md.

use crate::{
    check::Checker,
    types::{Ty, merge_inferred_types, needs_expected_type},
};
use fpas_parser::Expr;

impl Checker {
    /// Checks all array elements before completing unresolved element types.
    pub(super) fn check_array_literal(&mut self, elements: &[Expr], expected: Option<&Ty>) -> Ty {
        let expected = match expected {
            Some(Ty::Array(element)) => Some(element.as_ref()),
            _ => None,
        };
        let element = self.check_collection_values(
            &elements.iter().collect::<Vec<_>>(),
            expected,
            "array elements",
        );
        Ty::Array(Box::new(element))
    }

    /// Checks dictionary keys and values with their independently shared types.
    pub(super) fn check_dict_literal(
        &mut self,
        pairs: &[(Expr, Expr)],
        expected: Option<&Ty>,
    ) -> Ty {
        let (key_expected, value_expected) = match expected {
            Some(Ty::Dict(key, value)) => (Some(key.as_ref()), Some(value.as_ref())),
            _ => (None, None),
        };
        let keys = pairs.iter().map(|(key, _)| key).collect::<Vec<_>>();
        let values = pairs.iter().map(|(_, value)| value).collect::<Vec<_>>();
        let key = self.check_collection_values(&keys, key_expected, "dictionary keys");
        let value = self.check_collection_values(&values, value_expected, "dictionary values");
        if let Some((first, _)) = pairs.first() {
            self.defer_dictionary_literal_key_check(key.clone(), first.span());
        }
        Ty::Dict(Box::new(key), Box::new(value))
    }

    fn check_collection_values(
        &mut self,
        values: &[&Expr],
        expected: Option<&Ty>,
        role: &str,
    ) -> Ty {
        if values.is_empty() {
            return expected.cloned().unwrap_or(Ty::Error);
        }
        self.deferred_constructor_inference += 1;
        let mut types = values
            .iter()
            .map(|value| (self.check_expr_with_expected(value, expected), value.span()))
            .collect::<Vec<_>>();
        self.deferred_constructor_inference -= 1;
        let evidence = types
            .iter()
            .find(|(ty, _)| !needs_expected_type(ty))
            .map(|(ty, _)| ty.clone());
        if let Some(evidence) = &evidence {
            for (value, (actual, _)) in values.iter().zip(&mut types) {
                if !actual.is_error() && needs_expected_type(actual) {
                    *actual = self.check_expr_with_expected(value, Some(evidence));
                }
            }
        }
        let mut shared = self.shared_branch_type(types.clone(), role, None);
        if let Some(expected) = expected
            && shared.assignment_compatible_with(expected)
            && expected.assignment_compatible_with(&shared)
        {
            shared = merge_inferred_types(&shared, expected);
        }
        for (value, (actual, _)) in values.iter().zip(types) {
            if !actual.is_error() && needs_expected_type(&actual) {
                let context = (!needs_expected_type(&shared)).then_some(&shared);
                self.check_expr_with_expected(value, context);
            }
        }
        shared
    }
}
