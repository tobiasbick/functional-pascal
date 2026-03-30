use crate::Span;
use fpas_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticStage};

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
        DiagnosticStage::Lex,
        message,
        Some(hint.into()),
        span.into(),
    )
}

#[must_use]
pub fn lex_warning(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> LexError {
    Diagnostic::warning(
        code,
        DiagnosticStage::Lex,
        message,
        Some(hint.into()),
        span.into(),
    )
}
