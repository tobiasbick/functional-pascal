//! Generic declaration scopes and source parameter identities.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{GenericParamDef, GenericParameterId, Ty, TypeConstraint};
use fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE;
use fpas_lexer::Span;
use fpas_parser::TypeParam;
use std::sync::Arc;

impl Checker {
    /// Execute `f` with the given type parameters in scope, then pop the scope.
    pub(super) fn with_type_params<T>(
        &mut self,
        type_params: &[TypeParam],
        span: Span,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if !type_params.is_empty() {
            self.push_type_param_scope(type_params, span);
        }
        let previous_proofs = self.add_pure_parameter_proofs(type_params, true);
        let result = f(self);
        self.pure_parameters = previous_proofs;
        if !type_params.is_empty() {
            self.scopes.pop_scope();
        }
        result
    }

    /// Push a temporary scope with generic type parameters defined as `GenericParam`.
    /// Validates constraint names and reports errors for unknown constraints.
    pub(in crate::check::decl) fn push_type_param_scope(
        &mut self,
        type_params: &[TypeParam],
        span: Span,
    ) {
        self.scopes.push_scope();
        self.check_unique_type_param_names(type_params, span);
        for tp in type_params {
            let constraint = tp
                .constraint
                .as_ref()
                .and_then(|c| TypeConstraint::from_name(c));
            if tp.constraint.is_some() && constraint.is_none() {
                self.error_with_code(
                    SEMA_UNKNOWN_TYPE,
                    format!(
                        "Unknown type constraint `{}`",
                        tp.constraint.as_deref().unwrap_or("")
                    ),
                    "Valid constraints: Equatable, Comparable, Numeric, Printable.",
                    span,
                );
            }
            self.scopes.define(
                &tp.name,
                Symbol {
                    ty: Ty::GenericParam(Arc::new(self.resolve_type_param(tp))),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }
    }

    /// Convert AST type parameters to resolved `GenericParamDef`s.
    pub(in crate::check) fn resolve_type_params(
        &self,
        type_params: &[TypeParam],
    ) -> Vec<GenericParamDef> {
        type_params
            .iter()
            .map(|tp| self.resolve_type_param(tp))
            .collect()
    }

    /// Resolve a parameter consistently in header, body and recursive type scopes.
    pub(in crate::check) fn resolve_type_param(&self, parameter: &TypeParam) -> GenericParamDef {
        let unit = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.as_ref())
            .map(|name| name.to_ascii_lowercase());
        GenericParamDef {
            name: parameter.name.clone(),
            constraint: parameter
                .constraint
                .as_deref()
                .and_then(TypeConstraint::from_name),
            identity: GenericParameterId {
                source_id: if unit.is_some() {
                    0
                } else {
                    parameter.span.source_id
                },
                unit,
                offset: parameter.span.offset as u64,
            },
        }
    }
}
