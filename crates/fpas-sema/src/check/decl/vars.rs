use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_MISSING_RECORD_FIELD, SEMA_UNKNOWN_NAME,
};
use fpas_parser::{Expr, FieldInit, VarDef};

impl Checker {
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        let declared_ty = self.resolve_type_expr(&v.type_expr);

        let value_ty = self.check_expr_with_expected_record_literals(&v.value, &declared_ty);
        self.check_type_compat(&declared_ty, &value_ty, "variable initializer", v.span);

        let stored_ty = match (&declared_ty, self.ty_of_checked(&v.value)) {
            (crate::types::Ty::Task(inner), crate::types::Ty::Task(actual))
                if inner.is_error() && !actual.is_error() =>
            {
                crate::types::Ty::Task(actual.clone())
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

    fn validate_typed_record_literal_fields(
        &mut self,
        fields: &[FieldInit],
        record_ty: &crate::types::RecordTy,
        span: fpas_lexer::Span,
    ) {
        let construction_rejected = self.reject_private_record_construction(record_ty, span);

        for field_init in fields {
            if let Some((_, field_ty)) = record_ty
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
            {
                let value_ty =
                    self.check_expr_with_expected_record_literals(&field_init.value, field_ty);
                self.check_type_compat(
                    field_ty,
                    &value_ty,
                    &format!("field `{}`", field_init.name),
                    span,
                );
            } else {
                if record_ty
                    .properties
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
                {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!(
                            "Record type `{}` property `{}` cannot be initialized in a record literal",
                            record_ty.name, field_init.name
                        ),
                        "Properties are not record fields. Assign to the property after construction.",
                        span,
                    );
                } else if self
                    .find_record_event_on_type(record_ty, &field_init.name)
                    .is_some()
                {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!(
                            "Record type `{}` event `{}` cannot be initialized in a record literal",
                            record_ty.name, field_init.name
                        ),
                        "Events are not record fields. Assign a handler after construction.",
                        span,
                    );
                } else {
                    let known: Vec<&str> =
                        record_ty.fields.iter().map(|(n, _)| n.as_str()).collect();
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!(
                            "Record type `{}` has no field `{}`",
                            record_ty.name, field_init.name
                        ),
                        format!(
                            "Known fields: {}. Remove the unknown field or fix the name.",
                            known.join(", ")
                        ),
                        span,
                    );
                }
                // Still check sub-expressions to collect further errors.
                let _ = self.check_expr(&field_init.value);
            }
        }

        if construction_rejected {
            return;
        }

        // Check all required fields (those without a default) are provided.
        let provided: std::collections::HashSet<String> =
            fields.iter().map(|f| f.name.to_ascii_lowercase()).collect();
        let defaults = self
            .record_defaults
            .get(&record_ty.name)
            .cloned()
            .unwrap_or_default();

        for (field_name, _) in &record_ty.fields {
            if provided.contains(&field_name.to_ascii_lowercase()) {
                continue;
            }
            let has_default = defaults
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(field_name))
                .is_some_and(|(_, d)| d.is_some());
            if !has_default {
                self.error_with_code(
                    SEMA_MISSING_RECORD_FIELD,
                    format!(
                        "Required field `{field_name}` is missing from record literal for type `{}`",
                        record_ty.name
                    ),
                    format!(
                        "Provide `{field_name} := <value>`, or add a default to the field in the \
                         type definition: `{field_name}: <Type> := <default>;`."
                    ),
                    span,
                );
            }
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

    /// When a record or array-of-record expression is contextually typed, annotate it with
    /// the named record type so the compiler emits `MakeRecord` with the runtime type tag.
    pub(crate) fn check_expr_with_expected_record_literals(
        &mut self,
        expr: &Expr,
        expected: &Ty,
    ) -> Ty {
        self.try_annotate_expected_record_literals(expr, expected)
            .unwrap_or_else(|| self.check_expr(expr))
    }

    fn try_annotate_expected_record_literals(&mut self, expr: &Expr, expected: &Ty) -> Option<Ty> {
        let resolved = self.resolve_visible_type(expected);
        match (expr, &resolved) {
            (
                Expr::RecordLiteral {
                    fields,
                    span: lit_span,
                },
                Ty::Record(record_ty),
            ) => {
                self.validate_unique_record_fields(fields, "record literal");
                self.validate_typed_record_literal_fields(fields, record_ty, *lit_span);
                let key = Self::expr_lookup_key(expr);
                self.expr_types.insert(key, Ty::Record(record_ty.clone()));
                self.propagate_task_bound_expr(expr, key);
                Some(Ty::Record(record_ty.clone()))
            }
            (Expr::ArrayLiteral(elements, _), Ty::Array(element_ty)) => {
                let element_resolved = self.resolve_visible_type(element_ty);
                if matches!(element_resolved, Ty::Record(_)) {
                    for element in elements {
                        let actual =
                            self.check_expr_with_expected_record_literals(element, element_ty);
                        self.check_type_compat(
                            element_ty,
                            &actual,
                            "array element",
                            element.span(),
                        );
                    }
                    let key = Self::expr_lookup_key(expr);
                    self.expr_types.insert(key, resolved.clone());
                    self.propagate_task_bound_expr(expr, key);
                    Some(resolved)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
