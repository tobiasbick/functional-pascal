use fpas_diagnostics::{Diagnostic, DiagnosticCode};
use fpas_lexer::Span;

pub type SemaError = Diagnostic;

#[must_use]
pub fn sema_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> SemaError {
    Diagnostic::error(code, code.stage(), message, Some(hint.into()), span.into())
}
