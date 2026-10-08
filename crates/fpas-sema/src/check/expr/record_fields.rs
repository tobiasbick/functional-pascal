//! Shared validation for record literal and update field lists.

use super::Checker;
use crate::scope::canonical_symbol_name;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::FieldInit;
use std::collections::HashSet;

impl Checker {
    /// Report case-insensitive duplicate names in one record field initializer list.
    pub(crate) fn validate_unique_record_fields(&mut self, fields: &[FieldInit], context: &str) {
        self.validate_unique_record_field_names(
            fields.iter().map(|field| (field.name.as_str(), field.span)),
            context,
        );
    }

    /// Report duplicates for borrowed literal or constructor fields without cloning expressions.
    pub(crate) fn validate_unique_record_field_names<'a>(
        &mut self,
        fields: impl IntoIterator<Item = (&'a str, fpas_lexer::Span)>,
        context: &str,
    ) {
        let mut seen = HashSet::new();
        for (name, span) in fields {
            if seen.insert(canonical_symbol_name(name)) {
                continue;
            }
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Field `{name}` is specified more than once in {context}"),
                format!("Remove the duplicate `{name} := ...` entry."),
                span,
            );
        }
    }
}
