//! Shared validation for record literal and update field lists.

use super::Checker;
use crate::scope::canonical_symbol_name;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::FieldInit;
use std::collections::HashSet;

impl Checker {
    /// Report case-insensitive duplicate names in one record field initializer list.
    pub(crate) fn validate_unique_record_fields(&mut self, fields: &[FieldInit], context: &str) {
        let mut seen = HashSet::with_capacity(fields.len());
        for field in fields {
            if seen.insert(canonical_symbol_name(&field.name)) {
                continue;
            }
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!(
                    "Field `{}` is specified more than once in {context}",
                    field.name
                ),
                format!("Remove the duplicate `{} := ...` entry.", field.name),
                field.span,
            );
        }
    }
}
