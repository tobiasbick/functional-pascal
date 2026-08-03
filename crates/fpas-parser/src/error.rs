use fpas_diagnostics::{Diagnostic, DiagnosticCode};
use fpas_lexer::Span;

/// Parser-stage diagnostic returned by the public parse entry points.
pub type ParseError = Diagnostic;

#[must_use]
pub(crate) fn parse_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> ParseError {
    Diagnostic::error(
        code,
        message,
        Some(hint.into()),
        span.diagnostic_span_or_synthetic(),
    )
}
