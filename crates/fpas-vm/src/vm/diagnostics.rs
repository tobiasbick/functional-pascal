//! Instruction-address-based diagnostics with lazy sparse-source resolution.

use fpas_bytecode::{Executable, InstructionAddress};
use fpas_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan};

/// Structured runtime diagnostic returned by VM operations.
///
/// The diagnostic is boxed so `Result<_, VmError>` stays two words wide on hot interpreter paths.
pub type VmError = Box<Diagnostic>;

pub(crate) fn runtime_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
    location: fpas_bytecode::SourceLocation,
) -> VmError {
    Box::new(Diagnostic::error(
        code,
        message,
        Some(help.into()),
        SourceSpan::synthetic_from_location(location),
    ))
}

pub(crate) fn internal_error(
    message: impl Into<String>,
    help: impl Into<String>,
    location: fpas_bytecode::SourceLocation,
) -> VmError {
    runtime_error(
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        message,
        help,
        location,
    )
}

#[cold]
#[inline(never)]
pub(super) fn at_address(
    executable: &Executable,
    address: InstructionAddress,
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
) -> VmError {
    let mut diagnostic = Diagnostic::error_without_source(code, message, Some(help.into()));
    diagnostic.span = executable.source_map.lookup(address).map(|run| {
        SourceSpan::synthetic_from_location(fpas_diagnostics::SourceLocation::new_with_source(
            run.line,
            run.column,
            run.source.get(),
        ))
    });
    Box::new(diagnostic)
}

#[cold]
#[inline(never)]
pub(super) fn internal(
    executable: &Executable,
    address: InstructionAddress,
    message: impl Into<String>,
) -> VmError {
    at_address(
        executable,
        address,
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        message,
        "This indicates a compiler, verifier, or register-runtime invariant failure. Please report it.",
    )
}
