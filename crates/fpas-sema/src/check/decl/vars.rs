use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_MISSING_RECORD_FIELD, SEMA_UNKNOWN_NAME,
};
use fpas_parser::{Expr, FieldInit, VarDef};

impl Checker {
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        let declared_ty = self.resolve_type_expr(&v.type_expr);

        // Special handling: if the value is a record literal and the declared type resolves
        // to a concrete named record, validate field presence (including defaults) and
        // annotate the literal with the named type so the compiler can expand defaults.
        let skip_compat = if let Expr::RecordLiteral { fields, span } = &v.value {
            let resolved = self.resolve_visible_type(&declared_ty);
            if let crate::types::Ty::Record(record_ty) = resolved {
                self.validate_typed_record_literal(fields, &record_ty, *span);
                // Override the annotation so the compiler sees the named type.
                let key = Self::expr_lookup_key(&v.value);
                self.expr_types
                    .insert(key, crate::types::Ty::Record(record_ty));
                true
            } else {
                false
            }
        } else {
            false
        };

        if !skip_compat {
            let value_ty = self.check_expr(&v.value);
            self.check_type_compat(&declared_ty, &value_ty, "variable initializer", v.span);
        }

        let stored_ty = match (&declared_ty, self.ty_of_checked(&v.value)) {
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

    /// Validate a record literal against a concrete named record type.
    ///
    /// Checks that:
    /// - every provided field name exists in `record_ty`;
    /// - no field is specified more than once;
    /// - every required field (no default) is present in `fields`.
    ///
    /// Also type-checks each provided field value.
    pub(crate) fn validate_typed_record_literal(
        &mut self,
        fields: &[FieldInit],
        record_ty: &crate::types::RecordTy,
        span: fpas_lexer::Span,
    ) {
        // Check each provided field.
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for field_init in fields {
            // Duplicate field in literal.
            if !seen_names.insert(field_init.name.clone()) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!(
                        "Field `{}` is specified more than once in record literal",
                        field_init.name
                    ),
                    format!("Remove the duplicate `{} := …` entry.", field_init.name),
                    span,
                );
            }

            if let Some((_, field_ty)) = record_ty
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
            {
                let value_ty = self.check_expr(&field_init.value);
                self.check_type_compat(
                    field_ty,
                    &value_ty,
                    &format!("field `{}`", field_init.name),
                    span,
                );
            } else {
                let known: Vec<&str> = record_ty.fields.iter().map(|(n, _)| n.as_str()).collect();
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
                // Still check sub-expressions to collect further errors.
                let _ = self.check_expr(&field_init.value);
            }
        }

        // Check all required fields (those without a default) are provided.
        let provided: std::collections::HashSet<String> = fields
            .iter()
            .map(|f| f.name.to_ascii_lowercase())
            .collect();
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
}
