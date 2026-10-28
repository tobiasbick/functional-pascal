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

    pub(crate) fn from_session(error: fpas_runtime::execution::DebugSessionError) -> Self {
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
pub(crate) fn session_code(kind: fpas_runtime::execution::DebugErrorKind) -> &'static str {
    match kind {
        fpas_runtime::execution::DebugErrorKind::InvalidState => "invalid_state",
        fpas_runtime::execution::DebugErrorKind::UnknownTask => "unknown_task",
        fpas_runtime::execution::DebugErrorKind::UnknownBreakpoint => "unknown_breakpoint",
        fpas_runtime::execution::DebugErrorKind::BreakpointLimit => "breakpoint_limit",
        fpas_runtime::execution::DebugErrorKind::UnknownFrame => "unknown_frame",
        fpas_runtime::execution::DebugErrorKind::FrameRestartUnsupported => {
            "frame_restart_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::InstructionChangeUnsupported => {
            "instruction_change_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::TaskCreateUnsupported => "task_create_unsupported",
        fpas_runtime::execution::DebugErrorKind::TaskRestartUnsupported => {
            "task_restart_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::LiveInputUnsupported => "live_input_unsupported",
        fpas_runtime::execution::DebugErrorKind::DebuggeeInputLimit => "debuggee_input_limit",
        fpas_runtime::execution::DebugErrorKind::DebuggeeInputClosed => "debuggee_input_closed",
        fpas_runtime::execution::DebugErrorKind::FrameReturnUnsupported => {
            "frame_return_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::FrameReturnValueRequired => {
            "frame_return_value_required"
        }
        fpas_runtime::execution::DebugErrorKind::FrameReturnValueUnexpected => {
            "frame_return_value_unexpected"
        }
        fpas_runtime::execution::DebugErrorKind::FrameReturnType => "frame_return_type",
        fpas_runtime::execution::DebugErrorKind::TaskResultReplacementUnsupported => {
            "task_result_replacement_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::TaskResultReplacementType => {
            "task_result_replacement_type"
        }
        fpas_runtime::execution::DebugErrorKind::VariantUnknown => "variant_unknown",
        fpas_runtime::execution::DebugErrorKind::VariantFieldSet => "variant_field_set",
        fpas_runtime::execution::DebugErrorKind::StorageAlreadyInitialized => {
            "storage_already_initialized"
        }
        fpas_runtime::execution::DebugErrorKind::UnknownVariablesReference => {
            "unknown_variables_reference"
        }
        fpas_runtime::execution::DebugErrorKind::VariableTargetUnknown => "variable_target_unknown",
        fpas_runtime::execution::DebugErrorKind::VariableTargetExpired => "variable_target_expired",
        fpas_runtime::execution::DebugErrorKind::VariableNotMutable => "variable_not_mutable",
        fpas_runtime::execution::DebugErrorKind::VariablePathUnsupported => {
            "variable_path_unsupported"
        }
        fpas_runtime::execution::DebugErrorKind::VariableUninitialized => "variable_uninitialized",
        fpas_runtime::execution::DebugErrorKind::VariableValueType => "variable_value_type",
        fpas_runtime::execution::DebugErrorKind::VariableUnavailable => "variable_unavailable",
        fpas_runtime::execution::DebugErrorKind::DictionaryKeyExists => "dictionary_key_exists",
        fpas_runtime::execution::DebugErrorKind::DictionaryKeyMissing => "dictionary_key_missing",
        fpas_runtime::execution::DebugErrorKind::DictionaryKeyUnchanged => {
            "dictionary_key_unchanged"
        }
        fpas_runtime::execution::DebugErrorKind::SequenceIndexOutOfBounds => {
            "sequence_index_out_of_bounds"
        }
        fpas_runtime::execution::DebugErrorKind::StringCharacterRequired => {
            "string_character_required"
        }
        fpas_runtime::execution::DebugErrorKind::StringCharacterUnchanged => {
            "string_character_unchanged"
        }
        fpas_runtime::execution::DebugErrorKind::InspectionLimit => "limit_exceeded",
        fpas_runtime::execution::DebugErrorKind::UnknownName => "unknown_name",
        fpas_runtime::execution::DebugErrorKind::UninitializedValue => "uninitialized_value",
        fpas_runtime::execution::DebugErrorKind::EvaluationType => "evaluation_type",
        fpas_runtime::execution::DebugErrorKind::EvaluationDomain => "evaluation_domain",
        fpas_runtime::execution::DebugErrorKind::EvaluationLimit => "evaluation_limit",
        fpas_runtime::execution::DebugErrorKind::UnavailableValue => "unavailable_value",
        fpas_runtime::execution::DebugErrorKind::UnknownCallable => "call_target_unknown",
        fpas_runtime::execution::DebugErrorKind::AmbiguousCallable => "call_ambiguous",
        fpas_runtime::execution::DebugErrorKind::CallArity => "call_arity",
        fpas_runtime::execution::DebugErrorKind::ForbiddenCallEffect => "call_effect_forbidden",
        fpas_runtime::execution::DebugErrorKind::CallLimit => "call_limit",
        fpas_runtime::execution::DebugErrorKind::CallTimeout => "call_timeout",
        fpas_runtime::execution::DebugErrorKind::CallCancelled => "call_cancelled",
        fpas_runtime::execution::DebugErrorKind::CallRuntime => "call_runtime",
        fpas_runtime::execution::DebugErrorKind::ExecutionTimeout => "timeout",
        fpas_runtime::execution::DebugErrorKind::InstructionLimit => "instruction_limit",
        fpas_runtime::execution::DebugErrorKind::OutputLimit => "output_limit",
        fpas_runtime::execution::DebugErrorKind::ScheduleDeadlock => "schedule_deadlock",
        fpas_runtime::execution::DebugErrorKind::ScheduleClockLimit => "schedule_clock_limit",
        fpas_runtime::execution::DebugErrorKind::TraceEventLimit => "trace_event_limit",
        fpas_runtime::execution::DebugErrorKind::TraceValueLimit => "trace_value_limit",
        fpas_runtime::execution::DebugErrorKind::TraceWorkLimit => "trace_work_limit",
        fpas_runtime::execution::DebugErrorKind::TraceByteLimit => "trace_byte_limit",
        fpas_runtime::execution::DebugErrorKind::RecordingHostPath => "recording_host_path",
        fpas_runtime::execution::DebugErrorKind::LiveImageIncompatible => "live_image_incompatible",
        fpas_runtime::execution::DebugErrorKind::LiveImageBuildFailed => "live_image_build_failed",
        fpas_runtime::execution::DebugErrorKind::LiveImageRollbackUnavailable => {
            "live_image_rollback_unavailable"
        }
    }
}
