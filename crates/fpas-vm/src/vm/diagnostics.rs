//! Instruction-address-based diagnostics with lazy sparse-source resolution.

use fpas_bytecode::{Executable, InstructionAddress};
use fpas_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan};

/// Structured runtime diagnostic returned by VM operations.
pub type VmError = Diagnostic;

pub(crate) fn runtime_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
    location: fpas_bytecode::SourceLocation,
) -> VmError {
    Diagnostic::error(
        code,
        message,
        Some(help.into()),
        SourceSpan::new(0, 1, location.line(), location.column()),
    )
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

pub(super) fn at_address(
    executable: &Executable,
    address: InstructionAddress,
    code: DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
) -> Diagnostic {
    let span = executable.source_map.lookup(address).map_or_else(
        || SourceSpan::new(0, 1, 1, 1),
        |run| SourceSpan::new_with_source(0, 1, run.line, run.column, run.source.get()),
    );
    Diagnostic::error(code, message, Some(help.into()), span)
}

pub(super) fn internal(
    executable: &Executable,
    address: InstructionAddress,
    message: impl Into<String>,
) -> Diagnostic {
    at_address(
        executable,
        address,
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        message,
        "This indicates a compiler, verifier, or register-runtime invariant failure. Please report it.",
    )
}
