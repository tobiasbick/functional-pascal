//! Explicit pure evaluation contexts shared by routines and record defaults.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`.

pub(in crate::check) mod defaults;
mod reads;
mod signatures;

use crate::check::Checker;
use crate::types::{GenericParameterId, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use std::collections::HashSet;

/// Lexical boundary within which mutable local data belongs to this evaluation.
#[derive(Clone, Copy)]
pub(in crate::check) struct PureEvaluation {
    pub scope_index: usize,
}

impl Checker {
    /// Test the recursive data capability using visible nominal definitions and proofs.
    pub(crate) fn is_pure_data(&self, ty: &Ty) -> bool {
        self.record_default_purity_requirement(ty, false);
        ty.is_pure_data_with(|ty| {
            let resolved = self.resolve_visible_type(ty);
            if let Ty::GenericParam(parameter) = &resolved
                && self.pure_parameters.contains(&parameter.identity)
            {
                return Ty::Integer;
            }
            resolved
        })
    }

    pub(crate) fn reject_impure_operation(&mut self, operation: &str, span: Span) {
        if self.pure_evaluation.is_some() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Pure evaluation cannot {operation}"),
                "Use explicitly pure functions and immutable resource-free values.",
                span,
            );
        }
    }

    /// Retain only generic proofs required by explicitly pure signature positions.
    pub(in crate::check) fn retain_signature_proofs(
        &mut self,
        parameters: &[fpas_parser::TypeParam],
        params: &[crate::types::ParamTy],
        result: Option<&Ty>,
    ) {
        let mut required = Vec::new();
        for parameter in parameters {
            let identity = self.resolve_type_param(parameter).identity;
            self.pure_parameters.remove(&identity);
            if params
                .iter()
                .any(|p| !self.has_valid_pure_signatures(&p.ty))
                || result.is_some_and(|r| !self.has_valid_pure_signatures(r))
            {
                required.push(identity.clone());
            }
            self.pure_parameters.insert(identity);
        }
        for parameter in parameters {
            self.pure_parameters
                .remove(&self.resolve_type_param(parameter).identity);
        }
        self.pure_parameters.extend(required);
    }

    pub(in crate::check) fn add_pure_parameter_proofs(
        &mut self,
        parameters: &[fpas_parser::TypeParam],
        pure: bool,
    ) -> HashSet<GenericParameterId> {
        let previous = self.pure_parameters.clone();
        if pure {
            for parameter in parameters {
                self.pure_parameters
                    .insert(self.resolve_type_param(parameter).identity);
            }
        }
        previous
    }
}
