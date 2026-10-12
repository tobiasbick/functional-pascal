//! Deferred generic inference must finish before analysis permits lowering.
//! Documentation: docs/pascal/language/functions/generic-routines.md.

use super::Checker;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

/// Missing type arguments at a call or constructor awaiting an enclosing context.
pub(crate) struct PendingTypeArguments {
    message: String,
    hint: String,
    span: Span,
}

impl Checker {
    /// Clears a resolved site or retains its diagnostic until contextual checking finishes.
    pub(in crate::check) fn complete_or_defer_type_arguments(
        &mut self,
        name: &str,
        missing: Vec<String>,
        hint: &str,
        span: Span,
        may_defer: bool,
    ) {
        let key = (span.source_id, span.offset, name.to_ascii_lowercase());
        self.pending_type_arguments.remove(&key);
        if missing.is_empty() {
            return;
        }
        let message = format!(
            "Cannot infer type parameter(s) {} for `{name}`",
            missing.join(", ")
        );
        if may_defer && self.deferred_constructor_inference > 0 {
            self.pending_type_arguments.insert(
                key,
                PendingTypeArguments {
                    message,
                    hint: hint.to_owned(),
                    span,
                },
            );
        } else {
            self.error_with_code(SEMA_TYPE_MISMATCH, message, hint, span);
        }
    }

    /// Rejects every deferred site still unresolved after the complete program or unit check.
    pub(in crate::check) fn finish_deferred_type_arguments(&mut self) {
        for pending in std::mem::take(&mut self.pending_type_arguments).into_values() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                pending.message,
                pending.hint,
                pending.span,
            );
        }
    }
}
