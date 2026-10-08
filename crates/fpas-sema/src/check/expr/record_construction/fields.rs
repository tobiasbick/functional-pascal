//! Field checking for typed record construction.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::Checker;
use crate::types::RecordTy;
use fpas_diagnostics::codes::{SEMA_MISSING_RECORD_FIELD, SEMA_UNKNOWN_NAME};
use fpas_parser::Expr;

impl Checker {
    /// Check supplied values, visibility, and required fields of a typed construction.
    pub(crate) fn validate_typed_record_fields(
        &mut self,
        fields: &[(&str, &Expr, fpas_lexer::Span)],
        record_ty: &RecordTy,
        span: fpas_lexer::Span,
    ) {
        let construction_rejected = self.reject_private_record_construction(record_ty, span);

        for &(name, value, field_span) in fields {
            if let Some((_, field_ty)) = record_ty
                .fields
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            {
                let value_ty = self.check_expr(value);
                self.check_type_compat(
                    field_ty,
                    &value_ty,
                    &format!("field `{}`", name),
                    field_span,
                );
            } else {
                if self.find_record_event_on_type(record_ty, name).is_some() {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!(
                            "Record type `{}` event `{}` cannot be initialized in record construction",
                            record_ty.name, name
                        ),
                        "Events are not record fields. Assign a handler after construction.",
                        field_span,
                    );
                } else {
                    let known: Vec<&str> =
                        record_ty.fields.iter().map(|(n, _)| n.as_str()).collect();
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!("Record type `{}` has no field `{}`", record_ty.name, name),
                        format!(
                            "Known fields: {}. Remove the unknown field or fix the name.",
                            known.join(", ")
                        ),
                        field_span,
                    );
                }
                // Still check sub-expressions to collect further errors.
                let _ = self.check_expr(value);
            }
        }

        if construction_rejected {
            return;
        }

        // Check all required fields (those without a default) are provided.
        let provided: std::collections::HashSet<String> = fields
            .iter()
            .map(|(name, _, _)| name.to_ascii_lowercase())
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
