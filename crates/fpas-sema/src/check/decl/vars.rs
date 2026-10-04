use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{Expr, VarDef};

impl Checker {
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        let declared_ty = self.resolve_type_expr(&v.type_expr);

        let value_ty = self.check_expr_with_expected(&v.value, &declared_ty);
        self.check_type_compat(&declared_ty, &value_ty, "variable initializer", v.span);

        let stored_ty = match (&declared_ty, self.ty_of_checked(&v.value)) {
            (crate::types::Ty::Task(inner), crate::types::Ty::Task(actual))
                if inner.is_error() && !actual.is_error() =>
            {
                crate::types::Ty::Task(actual)
            }
            _ => declared_ty.clone(),
        };

        let task_bound = self.expr_is_task_bound(Self::expr_lookup_key(&v.value));
        if !self.scopes.define_with_declaration(
            &v.name,
            Symbol {
                ty: stored_ty,
                mutable,
                kind: SymbolKind::Var,
                task_bound,
            },
            v.span,
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate variable `{}`", v.name),
                "Each name must be unique in the same scope.",
                v.span,
            );
        }
    }

    /// Return the already-computed type for an expression (if annotated).
    fn ty_of_checked(&self, expr: &Expr) -> crate::types::Ty {
        let key = Self::expr_lookup_key(expr);
        self.expr_types
            .get(&key)
            .cloned()
            .unwrap_or(crate::types::Ty::Error)
    }
}
