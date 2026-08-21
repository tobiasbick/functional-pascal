//! Domain failures returned through the debug engine interface.

use super::DebugStatus;
use crate::evaluation::EvaluationParseError;

/// Failure of one debug engine request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EngineFailure {
    /// Stable failure code adapters map onto wire spelling.
    pub code: String,
    /// Human-readable failure.
    pub message: String,
    /// Corrective hint.
    pub help: String,
    /// Optional source offset for expression parse failures.
    pub offset: Option<usize>,
    /// Optional source length for expression parse failures.
    pub length: Option<usize>,
}

impl EngineFailure {
    pub(crate) fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            help: help.into(),
            offset: None,
            length: None,
        }
    }

    pub(crate) fn invalid_state(command: &str, status: DebugStatus) -> Self {
        Self::new(
            "invalid_state",
            format!("Command `{command}` is invalid while the protocol is {status:?}."),
            "Wait for the required lifecycle event before retrying.",
        )
    }

    pub(crate) fn unsupported_capability(command: &str) -> Self {
        Self::new(
            "unsupported_capability",
            format!("Debugger command `{command}` is not supported by protocol V2."),
            "Use a command advertised by `initialize`.",
        )
    }

    pub(crate) fn duplicate_request(id: u64) -> Self {
        Self::new(
            "invalid_request",
            format!("Request ID {id} was already used."),
            "Use a new positive request ID for every request.",
        )
    }

    pub(crate) fn terminated_session() -> Self {
        Self::new(
            "invalid_state",
            "The debugger session is terminated.",
            "Start a new `fpas debug` process.",
        )
    }

    pub(crate) fn from_session(error: fpas_vm::DebugSessionError) -> Self {
        Self::new(session_code(error.kind), error.message, error.hint)
    }

    pub(crate) fn from_parse(error: EvaluationParseError) -> Self {
        Self {
            code: error.code.to_string(),
            message: error.message,
            help: error.hint,
            offset: Some(error.offset),
            length: Some(error.length),
        }
    }
}

/// Stable JSONL/DAP failure code for a debug-session error kind.
#[must_use]
pub(crate) fn session_code(kind: fpas_vm::DebugErrorKind) -> &'static str {
    match kind {
        fpas_vm::DebugErrorKind::InvalidState => "invalid_state",
        fpas_vm::DebugErrorKind::UnknownTask => "unknown_task",
        fpas_vm::DebugErrorKind::UnknownBreakpoint => "unknown_breakpoint",
        fpas_vm::DebugErrorKind::BreakpointLimit => "breakpoint_limit",
        fpas_vm::DebugErrorKind::UnknownFrame => "unknown_frame",
        fpas_vm::DebugErrorKind::FrameRestartUnsupported => "frame_restart_unsupported",
        fpas_vm::DebugErrorKind::InstructionChangeUnsupported => "instruction_change_unsupported",
        fpas_vm::DebugErrorKind::TaskCreateUnsupported => "task_create_unsupported",
        fpas_vm::DebugErrorKind::TaskRestartUnsupported => "task_restart_unsupported",
        fpas_vm::DebugErrorKind::LiveInputUnsupported => "live_input_unsupported",
        fpas_vm::DebugErrorKind::DebuggeeInputLimit => "debuggee_input_limit",
        fpas_vm::DebugErrorKind::DebuggeeInputClosed => "debuggee_input_closed",
        fpas_vm::DebugErrorKind::FrameReturnUnsupported => "frame_return_unsupported",
        fpas_vm::DebugErrorKind::FrameReturnValueRequired => "frame_return_value_required",
        fpas_vm::DebugErrorKind::FrameReturnValueUnexpected => "frame_return_value_unexpected",
        fpas_vm::DebugErrorKind::FrameReturnType => "frame_return_type",
        fpas_vm::DebugErrorKind::TaskResultReplacementUnsupported => {
            "task_result_replacement_unsupported"
        }
        fpas_vm::DebugErrorKind::TaskResultReplacementType => "task_result_replacement_type",
        fpas_vm::DebugErrorKind::VariantUnknown => "variant_unknown",
        fpas_vm::DebugErrorKind::VariantFieldSet => "variant_field_set",
        fpas_vm::DebugErrorKind::StorageAlreadyInitialized => "storage_already_initialized",
        fpas_vm::DebugErrorKind::UnknownVariablesReference => "unknown_variables_reference",
        fpas_vm::DebugErrorKind::VariableTargetUnknown => "variable_target_unknown",
        fpas_vm::DebugErrorKind::VariableTargetExpired => "variable_target_expired",
        fpas_vm::DebugErrorKind::VariableNotMutable => "variable_not_mutable",
        fpas_vm::DebugErrorKind::VariablePathUnsupported => "variable_path_unsupported",
        fpas_vm::DebugErrorKind::VariableUninitialized => "variable_uninitialized",
        fpas_vm::DebugErrorKind::VariableValueType => "variable_value_type",
        fpas_vm::DebugErrorKind::VariableUnavailable => "variable_unavailable",
        fpas_vm::DebugErrorKind::DictionaryKeyExists => "dictionary_key_exists",
        fpas_vm::DebugErrorKind::DictionaryKeyMissing => "dictionary_key_missing",
        fpas_vm::DebugErrorKind::DictionaryKeyUnchanged => "dictionary_key_unchanged",
        fpas_vm::DebugErrorKind::SequenceIndexOutOfBounds => "sequence_index_out_of_bounds",
        fpas_vm::DebugErrorKind::StringCharacterRequired => "string_character_required",
        fpas_vm::DebugErrorKind::StringCharacterUnchanged => "string_character_unchanged",
        fpas_vm::DebugErrorKind::InspectionLimit => "limit_exceeded",
        fpas_vm::DebugErrorKind::UnknownName => "unknown_name",
        fpas_vm::DebugErrorKind::UninitializedValue => "uninitialized_value",
        fpas_vm::DebugErrorKind::EvaluationType => "evaluation_type",
        fpas_vm::DebugErrorKind::EvaluationDomain => "evaluation_domain",
        fpas_vm::DebugErrorKind::EvaluationLimit => "evaluation_limit",
        fpas_vm::DebugErrorKind::UnavailableValue => "unavailable_value",
        fpas_vm::DebugErrorKind::UnknownCallable => "call_target_unknown",
        fpas_vm::DebugErrorKind::AmbiguousCallable => "call_ambiguous",
        fpas_vm::DebugErrorKind::CallArity => "call_arity",
        fpas_vm::DebugErrorKind::ForbiddenCallEffect => "call_effect_forbidden",
        fpas_vm::DebugErrorKind::CallLimit => "call_limit",
        fpas_vm::DebugErrorKind::CallTimeout => "call_timeout",
        fpas_vm::DebugErrorKind::CallCancelled => "call_cancelled",
        fpas_vm::DebugErrorKind::CallRuntime => "call_runtime",
        fpas_vm::DebugErrorKind::ExecutionTimeout => "timeout",
        fpas_vm::DebugErrorKind::InstructionLimit => "instruction_limit",
        fpas_vm::DebugErrorKind::OutputLimit => "output_limit",
        fpas_vm::DebugErrorKind::RecordingHostPath => "recording_host_path",
        fpas_vm::DebugErrorKind::LiveImageIncompatible => "live_image_incompatible",
        fpas_vm::DebugErrorKind::LiveImageBuildFailed => "live_image_build_failed",
        fpas_vm::DebugErrorKind::LiveImageRollbackUnavailable => "live_image_rollback_unavailable",
    }
}
