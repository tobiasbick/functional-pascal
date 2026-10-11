use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{Expr, LocalBinding, TypeExpr, VarDef};

impl Checker {
    /// Checks an initializer and preserves its type and static capture guarantees.
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        self.check_binding(
            &v.name,
            Some(&v.type_expr),
            &v.value,
            v.span,
            mutable,
            false,
        );
    }

    /// Checks a statement-local binding and exposes its checked type to editor queries.
    ///
    /// **Documentation:** `docs/pascal/language/basics/local-variables.md`
    pub(crate) fn check_local_binding(&mut self, binding: &LocalBinding, mutable: bool) {
        let ty = self.check_binding(
            &binding.name,
            binding.type_expr.as_ref(),
            &binding.value,
            binding.span,
            mutable,
            !mutable,
        );
        if binding.type_expr.is_none() {
            self.binding_types
                .insert((binding.span.source_id, binding.span.offset), ty);
        }
    }

    /// Checks a binding while retaining expression identity for capture metadata.
    pub(crate) fn check_binding(
        &mut self,
        name: &str,
        type_expr: Option<&TypeExpr>,
        value: &Expr,
        span: fpas_lexer::Span,
        mutable: bool,
        constant: bool,
    ) -> crate::types::Ty {
        if self.check_import_alias_collision(name, span) {
            return crate::types::Ty::Error;
        }
        let declared_ty = if let Some(type_expr) = type_expr {
            let declared = self.resolve_type_expr(type_expr);
            let actual = self.check_expr_with_expected(value, Some(&declared));
            let context = if constant {
                "const initializer"
            } else {
                "variable initializer"
            };
            self.check_type_compat(&declared, &actual, context, span);
            declared
        } else {
            self.infer_local_binding(name, value, span, mutable)
        };

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
                record: compile_time
                    .then(|| self.record_constant_value(value))
                    .flatten(),
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
        stored_ty
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
