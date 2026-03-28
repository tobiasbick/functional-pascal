use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{TypeBody, TypeDef};

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
        let ty = self.with_type_params(&td.type_params, |checker| {
            checker.resolve_type_expr(type_expr)
        });
        self.define_type_symbol(td, ty);
    }

    pub(super) fn with_type_params<T>(
        &mut self,
        type_params: &[String],
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if !type_params.is_empty() {
            self.push_type_param_scope(type_params);
        }
        let result = f(self);
        if !type_params.is_empty() {
            self.scopes.pop_scope();
        }
        result
    }

    /// Push a temporary scope with generic type parameters defined as `GenericParam`.
    pub(super) fn push_type_param_scope(&mut self, type_params: &[String]) {
        self.scopes.push_scope();
        for type_param in type_params {
            self.scopes.define(
                type_param,
                Symbol {
                    ty: Ty::GenericParam(type_param.clone()),
                    mutable: false,
                    kind: SymbolKind::Type,
                },
            );
        }
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
