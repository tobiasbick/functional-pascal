use fpas_diagnostics::Diagnostic;
use fpas_diagnostics::codes::INTERNAL_COMPILER_INVARIANT_FAILURE;

/// Diagnostic returned when bytecode compilation fails.
pub type CompileError = Diagnostic;

#[must_use]
/// Creates an internal compiler diagnostic when only a source point is known.
pub fn internal_compiler_error(
    message: impl Into<String>,
    hint: impl Into<String>,
    line: u32,
    column: u32,
) -> CompileError {
    Diagnostic::error(
        INTERNAL_COMPILER_INVARIANT_FAILURE,
        message,
        Some(hint.into()),
        fpas_diagnostics::SourceSpan::synthetic_from_location(
            fpas_diagnostics::SourceLocation::new(line, column),
        ),
    )
}
