//! Contextual diagnostics for rejected anonymous record construction.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

impl Checker {
    /// Reject obsolete syntax with a resolved contextual type, when one exists.
    pub(super) fn reject_obsolete_record(&mut self, span: Span, expected: Option<&Ty>) -> Ty {
        let target = expected.map(|ty| self.resolve_visible_type(ty));
        let resolved_record = matches!(target, Some(Ty::Record(_)));
        let hint = if let Some(Ty::Record(record)) = target {
            let constructor = if record.name.contains('.') {
                "TypeName"
            } else {
                &record.name
            };
            format!(
                "Resolved record type: `{}`. Construct it through its visible type name or import alias with `{constructor}(Field := Value, ...)`.",
                record.name
            )
        } else {
            "Use a declared record type with named fields: `TypeName(Field := Value, ...)`. No record target can be resolved from this context.".into()
        };
        // Call inference may revisit the same operand with a more concrete expected type.
        if let Some(previous) = self.errors.iter_mut().find(|error| {
            error.message == "Anonymous record literals are obsolete"
                && error.span.is_some_and(|previous| {
                    previous.offset() == span.offset && previous.source_id() == span.source_id
                })
        }) {
            if resolved_record {
                previous.help = Some(hint);
            }
            return Ty::Error;
        }
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "Anonymous record literals are obsolete",
            hint,
            span,
        );
        Ty::Error
    }
}
