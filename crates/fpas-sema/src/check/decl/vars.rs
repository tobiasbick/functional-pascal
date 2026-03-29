use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::VarDef;

impl Checker {
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        let declared_ty = self.resolve_type_expr(&v.type_expr);
        let value_ty = self.check_expr(&v.value);
        self.check_type_compat(&declared_ty, &value_ty, "variable initializer", v.span);
        let stored_ty = match (&declared_ty, &value_ty) {
            (crate::types::Ty::Task(inner), crate::types::Ty::Task(actual))
                if inner.is_error() && !actual.is_error() =>
            {
                crate::types::Ty::Task(actual.clone())
            }
            _ => declared_ty.clone(),
        };

        if !self.scopes.define(
            &v.name,
            Symbol {
                ty: stored_ty,
                mutable,
                kind: SymbolKind::Var,
            },
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate variable `{}`", v.name),
                "Each name must be unique in the same scope.",
                v.span,
            );
        }
    }
}
