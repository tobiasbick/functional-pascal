//! Errors produced by standard library runtime (`Std.*` intrinsics and I/O).

use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;
use fpas_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticStage, SourceLocation, SourceSpan};

pub type StdError = Diagnostic;

#[must_use]
pub fn std_runtime_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> StdError {
    Diagnostic::error(
        code,
        DiagnosticStage::Runtime,
        message,
        Some(help.into()),
        SourceSpan::new(0, 1, location.line, location.column),
    )
}

#[must_use]
pub fn std_internal_error(
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> StdError {
    Diagnostic::error(
        INTERNAL_VM_INVARIANT_FAILURE,
        DiagnosticStage::Internal,
        message,
        Some(help.into()),
        SourceSpan::new(0, 1, location.line, location.column),
    )
}
