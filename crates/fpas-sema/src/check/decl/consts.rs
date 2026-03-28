use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::ConstDef;

impl Checker {
    pub(super) fn check_const_def(&mut self, c: &ConstDef) {
        let declared_ty = self.resolve_type_expr(&c.type_expr);
        let value_ty = self.check_expr(&c.value);
        self.check_type_compat(&declared_ty, &value_ty, "const initializer", c.span);

        if !self.scopes.define(
            &c.name,
            Symbol {
                ty: declared_ty,
                mutable: false,
                kind: SymbolKind::Const,
            },
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate constant `{}`", c.name),
                "Each name must be unique in the same scope.",
                c.span,
            );
        }
    }
}
