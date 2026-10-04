//! Field and default validation for declared record construction.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use crate::check::Checker;
use fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD;
use fpas_parser::FieldInit;

impl Checker {
    /// Check concrete field values, visibility and required defaults after inference.
    pub(in crate::check) fn validate_record_construction_fields(
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
                let value_ty = self.check_expr_with_expected(&field_init.value, field_ty);
                self.check_type_compat(
                    field_ty,
                    &value_ty,
                    &format!("field `{}`", field_init.name),
                    span,
                );
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
                        "Required field `{field_name}` is missing from record construction for type `{}`",
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
}
