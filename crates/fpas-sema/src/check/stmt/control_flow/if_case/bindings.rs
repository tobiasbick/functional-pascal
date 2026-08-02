//! Binding consistency for labels that share one `case` arm.

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

type CaseBindings = Vec<(String, Ty)>;

impl Checker {
    /// Return the common binding signature for one arm after validating every label.
    pub(super) fn shared_case_arm_bindings(
        &mut self,
        binding_sets: Vec<CaseBindings>,
        span: Span,
    ) -> CaseBindings {
        let Some(shared) = binding_sets
            .iter()
            .find(|bindings| !bindings.is_empty())
            .cloned()
        else {
            return Vec::new();
        };

        if binding_sets
            .iter()
            .any(|bindings| !binding_signatures_match(&shared, bindings))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Labels in the same case arm must bind the same names with compatible types",
                "Use separate case arms when patterns expose different names or payload types.",
                span,
            );
        }

        shared
    }
}

fn binding_signatures_match(expected: &CaseBindings, actual: &CaseBindings) -> bool {
    expected.len() == actual.len()
        && expected.iter().all(|(expected_name, expected_ty)| {
            actual.iter().any(|(actual_name, actual_ty)| {
                canonical_symbol_name(expected_name) == canonical_symbol_name(actual_name)
                    && expected_ty.compatible_with(actual_ty)
                    && actual_ty.compatible_with(expected_ty)
            })
        })
}
