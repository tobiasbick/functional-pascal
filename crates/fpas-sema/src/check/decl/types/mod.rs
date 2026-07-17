use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, GenericParamDef, Ty, TypeConstraint};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_UNKNOWN_TYPE};
use fpas_lexer::Span;
use fpas_parser::{TypeBody, TypeDef, TypeParam};

mod enums;
mod records;

impl Checker {
    pub(super) fn check_type_def(&mut self, td: &TypeDef) {
        match &td.body {
            TypeBody::Record(record) => self.check_record_type_def(td, record),
            TypeBody::Enum(enum_ty) => self.check_enum_type_def(td, enum_ty),
            TypeBody::Alias(type_expr) => self.check_alias_type_def(td, type_expr),
        }
    }

    fn check_alias_type_def(&mut self, td: &TypeDef, type_expr: &fpas_parser::TypeExpr) {
        let ty = self.resolve_type_expr(type_expr);
        if !self.define_type_symbol(td, ty.clone()) {
            return;
        }

        if let Ty::Enum(enum_ty) = ty {
            self.register_enum_alias_variant_symbols(td, &enum_ty);
        }
    }

    /// Expose qualified enum variants through an alias without adding ambiguous short names.
    fn register_enum_alias_variant_symbols(&mut self, td: &TypeDef, enum_ty: &EnumTy) {
        for variant in &enum_ty.variants {
            let kind = if variant.fields.is_empty() {
                SymbolKind::EnumMember
            } else {
                SymbolKind::EnumVariantConstructor
            };
            let qualified = format!("{}.{}", td.name, variant.name);
            if !self.scopes.define_in_root(
                &qualified,
                Symbol {
                    ty: Ty::Enum(enum_ty.clone()),
                    mutable: false,
                    kind,
                },
            ) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate enum member `{qualified}`"),
                    "Each enum member name must be unique in the program.",
                    td.span,
                );
            }
        }
    }

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
        let result = f(self);
        if !type_params.is_empty() {
            self.scopes.pop_scope();
        }
        result
    }

    /// Push a temporary scope with generic type parameters defined as `GenericParam`.
    /// Validates constraint names and reports errors for unknown constraints.
    pub(super) fn push_type_param_scope(&mut self, type_params: &[TypeParam], span: Span) {
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
                    "Valid constraints: Comparable, Numeric, Printable.",
                    span,
                );
            }
            self.scopes.define(
                &tp.name,
                Symbol {
                    ty: Ty::GenericParam(tp.name.clone(), constraint),
                    mutable: false,
                    kind: SymbolKind::Type,
                },
            );
        }
    }

    /// Convert AST type parameters to resolved `GenericParamDef`s.
    pub(super) fn resolve_type_params(type_params: &[TypeParam]) -> Vec<GenericParamDef> {
        type_params
            .iter()
            .map(|tp| GenericParamDef {
                name: tp.name.clone(),
                constraint: tp
                    .constraint
                    .as_ref()
                    .and_then(|c| TypeConstraint::from_name(c)),
            })
            .collect()
    }

    pub(super) fn define_type_symbol(&mut self, td: &TypeDef, ty: Ty) -> bool {
        if self.scopes.define(
            &td.name,
            Symbol {
                ty,
                mutable: false,
                kind: SymbolKind::Type,
            },
        ) {
            return true;
        }

        self.error_with_code(
            SEMA_DUPLICATE_DECLARATION,
            format!("Duplicate type `{}`", td.name),
            "Each name must be unique in the same scope.",
            td.span,
        );
        false
    }
}
