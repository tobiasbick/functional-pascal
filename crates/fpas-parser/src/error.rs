use fpas_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticStage};
use fpas_lexer::Span;

pub type ParseError = Diagnostic;

#[must_use]
pub fn parse_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> ParseError {
    Diagnostic::error(
        code,
        DiagnosticStage::Parse,
        message,
        Some(hint.into()),
        span.into(),
    )
}
