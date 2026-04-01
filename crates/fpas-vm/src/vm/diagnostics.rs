use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::{
    INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_INTRINSIC_STACK_STATE_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
use fpas_diagnostics::{Diagnostic, DiagnosticStage, SourceSpan};

pub type VmError = Diagnostic;

pub(crate) fn runtime_error(
    code: fpas_diagnostics::DiagnosticCode,
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> VmError {
    Diagnostic::error(
        code,
        DiagnosticStage::Runtime,
        message,
        Some(help.into()),
        SourceSpan::new_with_source(0, 1, location.line, location.column, location.source_id),
    )
}

pub(crate) fn internal_error(
    message: impl Into<String>,
    help: impl Into<String>,
    location: SourceLocation,
) -> VmError {
    Diagnostic::error(
        INTERNAL_VM_INVARIANT_FAILURE,
        DiagnosticStage::Internal,
        message,
        Some(help.into()),
        SourceSpan::new_with_source(0, 1, location.line, location.column, location.source_id),
    )
}

pub(super) const STACK_OVERFLOW_CODE: fpas_diagnostics::DiagnosticCode =
    RUNTIME_INTRINSIC_STACK_STATE_ERROR;
pub(super) const TYPE_MISMATCH_CODE: fpas_diagnostics::DiagnosticCode =
    RUNTIME_VM_OPERAND_TYPE_MISMATCH;
