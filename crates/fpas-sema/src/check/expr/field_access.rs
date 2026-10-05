//! Stored record field access, including fields containing callable values.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;

impl Checker {
    /// Resolve a stored field after checking record visibility.
    pub(crate) fn check_record_field_access(&mut self, ty: &Ty, field: &str, span: Span) -> Ty {
        if ty.is_error() {
            return Ty::Error;
        }
        let Ty::Record(record) = ty else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`.{field}` requires a record value"),
                self.field_access_hint(field, "Use field access on a record value."),
                span,
            );
            return Ty::Error;
        };
        if self.reject_private_record_member(record, field, span) {
            return Ty::Error;
        }
        if let Some((_, ty)) = record
            .fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(field))
        {
            return ty.clone();
        }
        self.error_with_code(
            SEMA_UNKNOWN_NAME,
            format!("Record `{}` has no field `{field}`", record.name),
            self.field_access_hint(field, "Check the field name against the record type."),
            span,
        );
        Ty::Error
    }

    // Teaches the ordinary call form when `.Name` names a visible routine instead of a field.
    fn field_access_hint(&self, field: &str, fallback: &str) -> String {
        let names_routine = self.scopes.lookup(field).is_some_and(|symbol| {
            matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure)
        });
        if names_routine {
            format!(
                "Values have no methods or receiver calls. Pass the value as an argument instead, for example `{field}(Value)`."
            )
        } else {
            fallback.to_owned()
        }
    }
}
