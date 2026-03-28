use fpas_diagnostics::codes::INTERNAL_COMPILER_INVARIANT_FAILURE;
use fpas_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticStage};
use fpas_lexer::Span;

pub type CompileError = Diagnostic;

#[must_use]
pub fn compile_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    hint: impl Into<String>,
    span: Span,
) -> CompileError {
    Diagnostic::error(
        code,
        DiagnosticStage::Compile,
        message,
        Some(hint.into()),
        span.into(),
    )
}

#[must_use]
pub fn internal_compiler_error(
    message: impl Into<String>,
    hint: impl Into<String>,
    line: u32,
    column: u32,
) -> CompileError {
    Diagnostic::error(
        INTERNAL_COMPILER_INVARIANT_FAILURE,
        DiagnosticStage::Internal,
        message,
        Some(hint.into()),
        fpas_diagnostics::SourceSpan::new(0, 0, line, column),
    )
}
