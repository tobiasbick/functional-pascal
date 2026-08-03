use crate::Span;
use fpas_diagnostics::{Diagnostic, DiagnosticCode};

pub type LexError = Diagnostic;

#[must_use]
pub fn lex_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> LexError {
    Diagnostic::error(
        code,
        message,
        Some(hint.into()),
        span.diagnostic_span_or_synthetic(),
    )
}
