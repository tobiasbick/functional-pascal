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

        let stored_ty = declared_ty;

        let task_bound = self.expr_is_task_bound(Self::expr_lookup_key(value));
        let constant_info = constant.then(|| self.binding_constant_info(value, &stored_ty));
        let symbol = Symbol {
            constant: constant_info,
            ty: stored_ty.clone(),
            mutable,
            kind: if constant {
                SymbolKind::Const
            } else {
                SymbolKind::Var
            },
            task_bound,
        };
        let installed = if self
            .predeclared_bindings
            .remove(&(span.source_id, span.offset))
        {
            if let Some(existing) = self.scopes.lookup_mut(name) {
                *existing = symbol;
            }
            true
        } else {
            self.scopes.define_with_declaration(name, symbol, span)
        };
        if !installed {
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

    /// Classify static inputs without evaluating or reordering their runtime initializers.
    pub(crate) fn binding_constant_info(
        &mut self,
        value: &Expr,
        ty: &crate::types::Ty,
    ) -> crate::scope::ConstantInfo {
        let compile_time = self.const_initializer_is_compile_time_known(value, ty);
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
    }
}
