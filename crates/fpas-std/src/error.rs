//! Errors produced by standard library runtime (`Std.*` intrinsics and I/O).
//!
//! Call-site [`SourceLocation`] is mapped to a [`SourceSpan`] with placeholder
//! `offset` and `length` (`0` and `1`) because std intrinsics only receive line,
//! column, and `source_id` from the VM, not byte offsets into source text.

use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;
use fpas_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticStage, SourceLocation, SourceSpan};

pub type StdError = Diagnostic;

#[must_use]
fn synthetic_span(location: SourceLocation) -> SourceSpan {
    SourceSpan::new_with_source(0, 1, location.line, location.column, location.source_id)
}

/// Runtime error with optional help line; pass `None` when the message alone is sufficient.
#[must_use]
pub fn std_runtime_error_opt(
    code: DiagnosticCode,
    message: impl Into<String>,
    help: Option<String>,
    location: SourceLocation,
) -> StdError {
    Diagnostic::error(
        code,
        DiagnosticStage::Runtime,
        message.into(),
        help,
        synthetic_span(location),
    )
}

/// Runtime error including a `help:` line; see [`std_runtime_error_opt`] to omit help.
#[must_use]
pub fn std_runtime_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> StdError {
    std_runtime_error_opt(code, message, Some(help.into()), location)
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
        message.into(),
        Some(help.into()),
        synthetic_span(location),
    )
}
