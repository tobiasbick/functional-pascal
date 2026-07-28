//! Record field and routine visibility enforcement.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::Checker;
use crate::types::RecordTy;
use fpas_diagnostics::codes::SEMA_PRIVATE_RECORD_MEMBER;
use fpas_lexer::Span;

impl Checker {
    pub(crate) fn record_member_is_visible(&self, record: &RecordTy, member: &str) -> bool {
        if !record
            .private_members
            .iter()
            .any(|name| name.eq_ignore_ascii_case(member))
        {
            return true;
        }
        self.current_owner_unit() == record.owner_unit.as_deref()
    }

    pub(crate) fn reject_private_record_member(
        &mut self,
        record: &RecordTy,
        member: &str,
        span: Span,
    ) -> bool {
        if self.record_member_is_visible(record, member) {
            return false;
        }
        let hint = record.owner_unit.as_deref().map_or_else(
            || "Use a public field or record routine.".to_string(),
            |owner| {
                format!(
                    "Access `{member}` only inside unit `{owner}`, or expose a public record routine."
                )
            },
        );
        self.error_with_code(
            SEMA_PRIVATE_RECORD_MEMBER,
            format!("Record member `{}.{member}` is private", record.name),
            hint,
            span,
        );
        true
    }

    pub(crate) fn reject_private_record_construction(
        &mut self,
        record: &RecordTy,
        span: Span,
    ) -> bool {
        let has_private_field = record.fields.iter().any(|(field, _)| {
            record
                .private_members
                .iter()
                .any(|private| private.eq_ignore_ascii_case(field))
        });
        if !has_private_field || self.current_owner_unit() == record.owner_unit.as_deref() {
            return false;
        }
        let hint = record.owner_unit.as_deref().map_or_else(
            || "Use a public function that returns the record type.".to_string(),
            |owner| {
                format!(
                    "Construct `{}` only inside unit `{owner}`, or call a public function that returns it.",
                    record.name
                )
            },
        );
        self.error_with_code(
            SEMA_PRIVATE_RECORD_MEMBER,
            format!(
                "Record type `{}` cannot be constructed here because it has private fields",
                record.name
            ),
            hint,
            span,
        );
        true
    }

    fn current_owner_unit(&self) -> Option<&str> {
        self.scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.as_deref())
    }
}
