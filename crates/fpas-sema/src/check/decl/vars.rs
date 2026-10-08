use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{Expr, TypeExpr, VarDef};

impl Checker {
    /// Checks an initializer and preserves its type and static capture guarantees.
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        self.check_binding(&v.name, &v.type_expr, &v.value, v.span, mutable, false);
    }

    /// Checks a typed binding while retaining expression identity for capture metadata.
    pub(crate) fn check_binding(
        &mut self,
        name: &str,
        type_expr: &TypeExpr,
        value: &Expr,
        span: fpas_lexer::Span,
        mutable: bool,
        constant: bool,
    ) {
        if self.check_import_alias_collision(name, span) {
            return;
        }
        let declared_ty = self.resolve_type_expr(type_expr);

        let value_ty = self.check_expr(value);
        let context = if constant {
            "const initializer"
        } else {
            "variable initializer"
        };
        self.check_type_compat(&declared_ty, &value_ty, context, span);

        let stored_ty = match (&declared_ty, self.ty_of_checked(value)) {
            (crate::types::Ty::Task(inner), crate::types::Ty::Task(actual))
                if inner.is_error() && !actual.is_error() =>
            {
                crate::types::Ty::Task(actual)
            }
            _ => declared_ty.clone(),
        };

        let task_bound = self.expr_is_task_bound(Self::expr_lookup_key(value));
        let constant_info = constant.then(|| {
            let compile_time = self.const_initializer_is_compile_time_known(value, &stored_ty);
            crate::scope::ConstantInfo {
                compile_time,
                value: if compile_time {
                    self.scalar_constant_value(value)
                } else {
                    None
                },
            }
        });
        if !self.scopes.define_with_declaration(
            name,
            Symbol {
                constant: constant_info,
                ty: stored_ty.clone(),
                mutable,
                kind: if constant {
                    SymbolKind::Const
                } else {
                    SymbolKind::Var
                },
                task_bound,
            },
            span,
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!(
                    "Duplicate {} `{name}`",
                    if constant { "constant" } else { "variable" }
                ),
                "Each name must be unique in the same scope.",
                span,
            );
        }
        self.record_binding_discard_info(name, &stored_ty, mutable, Some(value));
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
