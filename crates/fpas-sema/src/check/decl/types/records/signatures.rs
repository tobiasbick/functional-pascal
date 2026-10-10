//! Record method signature and body checking.

use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{ParamTy, Ty, TypeConstraint};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{FuncBody, TypeParam};

impl Checker {
    /// Require an instance record routine to declare the canonical `Self` receiver first.
    pub(super) fn validate_record_method_signature(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: &[ParamTy],
        span: fpas_lexer::Span,
    ) -> bool {
        let Some(self_param) = params.first() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        };

        if !self_param.name.eq_ignore_ascii_case("Self")
            || !matches!(&self_param.ty, Ty::Record(record) if record.name.eq_ignore_ascii_case(type_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        }

        true
    }

    /// Validate receiver rules shared by static record functions and procedures.
    pub(super) fn validate_static_routine_signature(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: &[ParamTy],
        span: fpas_lexer::Span,
        routine_kind: &str,
    ) -> bool {
        if params
            .iter()
            .any(|param| param.name.eq_ignore_ascii_case("Self"))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Static record {routine_kind} `{type_name}.{method_name}` must not declare a `Self` parameter"
                ),
                format!(
                    "Remove `Self` and call the {routine_kind} through the type: `{type_name}.{method_name}(...)`."
                ),
                span,
            );
            return false;
        }
        true
    }

    /// Check a deferred method body in its resolved generic, parameter, and owner context.
    pub(super) fn check_method_body(
        &mut self,
        qualified_name: &str,
        type_params: &[TypeParam],
        params: &[ParamTy],
        param_spans: &[fpas_lexer::Span],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) {
        let FuncBody::Block { nested, stmts } = body;
        self.discard_results.push(true);

        self.scopes.push_scope();

        // Introduce method-level generic type parameters as `GenericParam` types
        // so that expressions in the body can reference them.
        for type_param in type_params {
            let constraint = type_param
                .constraint
                .as_ref()
                .and_then(|constraint| TypeConstraint::from_name(constraint));
            self.scopes.define(
                &type_param.name,
                Symbol {
                    constant: None,
                    ty: Ty::GenericParam(type_param.name.clone(), constraint),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }

        for (param, span) in params.iter().zip(param_spans) {
            self.scopes.define_with_declaration(
                &param.name,
                Symbol {
                    constant: None,
                    ty: param.ty.clone(),
                    mutable: param.is_var(),
                    kind: SymbolKind::Param,
                    task_bound: false,
                },
                *span,
            );
            self.record_binding_discard_info(&param.name, &param.ty, false, None);
        }
        let previous_ctx = self.scopes.function_ctx.take();
        let owner_unit = previous_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.clone())
            .or_else(|| {
                qualified_name.rsplit_once('.').and_then(|(type_name, _)| {
                    type_name.rsplit_once('.').map(|(unit, _)| unit.to_string())
                })
            });
        self.scopes.function_ctx = Some(FunctionCtx {
            name: qualified_name.to_string(),
            return_type,
            owner_unit,
        });
        for decl in nested {
            self.check_decl(decl);
        }
        for stmt in stmts {
            self.check_stmt(stmt);
        }
        self.scopes.function_ctx = previous_ctx;
        self.scopes.pop_scope();
        self.scopes.set_discard_info(
            qualified_name,
            fpas_unit::interface::DiscardInfo {
                value: true,
                result: self.discard_results.pop().unwrap_or(false),
            },
        );
    }

    /// Resolve a method parameter type, treating the type name as the record type (for `Self`).
    pub(super) fn resolve_method_param_type(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
        type_name: &str,
        record_ty: &Ty,
    ) -> Ty {
        let resolved = self.resolve_type_expr(type_expr);
        if let Ty::Named(name) = &resolved
            && name.eq_ignore_ascii_case(type_name)
        {
            return record_ty.clone();
        }
        resolved
    }
}
