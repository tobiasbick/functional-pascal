//! Typed debugger results shared by protocol adapters.

use super::command::DebugCommand;
use super::error::EngineFailure;

/// A debugger result emitted after one typed engine request.
#[derive(Debug, Clone)]
pub(crate) enum DebugRecord {
    /// Completion of one request.
    Response {
        /// Adapter correlation identifier.
        request_id: u64,
        /// Completed debugger operation.
        command: DebugCommand,
        /// Success body or domain failure.
        outcome: Result<ResponseBody, EngineFailure>,
    },
    /// An asynchronous debugger event.
    Event(DebugEvent),
}

/// Successful engine payload for one command.
#[derive(Debug, Clone)]
pub(crate) enum ResponseBody {
    Accepted,
    Requested,
    Cancelled {
        cancelled: bool,
    },
    TerminatedAck,
    Eof,
    Cleared,
    Initialize {
        execution: fpas_runtime::execution::DebugExecutionLimits,
        hot_reload: bool,
    },
    Evaluate(fpas_runtime::execution::DebugEvaluateResult),
    Tasks {
        tasks: Vec<fpas_runtime::execution::DebugTask>,
        total: usize,
    },
    TaskHold {
        task_id: u64,
        paused: bool,
    },
    TaskCancelled {
        task_id: u64,
    },
    Stack {
        frames: Vec<fpas_runtime::execution::DebugFrame>,
        total: usize,
        task_id: u64,
    },
    Scopes {
        scopes: Vec<fpas_runtime::execution::DebugScope>,
    },
    Variables {
        variables: Vec<fpas_runtime::execution::DebugVariable>,
        total: usize,
    },
    InputQueued {
        bytes: usize,
        session_bytes: usize,
    },
    Breakpoint(fpas_runtime::execution::BoundBreakpoint),
    UnverifiedBreakpoint {
        source: String,
        line: u32,
        column: Option<u32>,
        message: String,
        error_code: String,
        error_offset: usize,
        error_length: usize,
    },
    BreakpointCleared {
        breakpoint_id: u64,
    },
    FunctionBreakpoints {
        breakpoints: Vec<fpas_runtime::execution::BoundFunctionBreakpoint>,
    },
    DataBreakpoints {
        breakpoints: Vec<fpas_runtime::execution::BoundDataBreakpoint>,
    },
    RuntimeFilters {
        filters: Vec<String>,
    },
    Dictionary(fpas_runtime::execution::DebugDictionaryMutationResult),
    Array(fpas_runtime::execution::DebugArrayMutationResult),
    StringCharacter(fpas_runtime::execution::DebugStringMutationResult),
    ForcedReturn(fpas_runtime::execution::DebugForcedReturnResult),
    FrameRestart(fpas_runtime::execution::DebugFrameRestartResult),
    Location(fpas_runtime::execution::DebugDataLocation),
    Recording {
        envelope: fpas_runtime::execution::DebugRecordingEnvelope,
        capturing: bool,
        events: Vec<fpas_runtime::execution::DebugRecordingEvent>,
        truncated: bool,
    },
    RecordingStarted {
        capturing: bool,
        truncated: bool,
        event_count: usize,
    },
    LiveImage {
        class: fpas_runtime::execution::LiveImageUpdateClass,
        accepted: bool,
        applied: bool,
        version: u64,
        rollback_available: bool,
    },
    VariantDescription {
        target: String,
        description: fpas_runtime::execution::DebugVariantDescription,
    },
    VariantConstruct(fpas_runtime::execution::DebugVariantConstructionResult),
    Storage(fpas_runtime::execution::DebugStorageInitializationResult),
    TaskResult(fpas_runtime::execution::DebugTaskResultReplacement),
}

/// Engine-owned debugger event.
#[derive(Debug, Clone)]
pub(crate) enum DebugEvent {
    Initialized,
    Stopped(fpas_runtime::execution::DebugStop),
    Task(fpas_runtime::execution::DebugTaskEvent),
    Output {
        category: &'static str,
        text: String,
        sequence: Option<usize>,
        breakpoint_id: Option<u64>,
        location: Option<fpas_runtime::execution::SourceLocation>,
    },
    Terminated {
        reason: &'static str,
        exit_code: i32,
        diagnostic_code: Option<String>,
        instruction_count: Option<u64>,
    },
    RuntimeError {
        diagnostic: fpas_diagnostics::Diagnostic,
        task_id: u64,
    },
    ProtocolError(EngineFailure),
    SourceBreakpoint(fpas_runtime::execution::BoundBreakpoint),
    FunctionBreakpoint(fpas_runtime::execution::BoundFunctionBreakpoint),
    DataBreakpoint(fpas_runtime::execution::BoundDataBreakpoint),
}

impl DebugRecord {
    #[must_use]
    pub(crate) fn ok(request_id: u64, command: DebugCommand, body: ResponseBody) -> Self {
        Self::Response {
            request_id,
            command,
            outcome: Ok(body),
        }
    }

    #[must_use]
    pub(crate) fn fail(request_id: u64, command: DebugCommand, error: EngineFailure) -> Self {
        Self::Response {
            request_id,
            command,
            outcome: Err(error),
        }
    }

    #[must_use]
    pub(crate) fn event(event: DebugEvent) -> Self {
        Self::Event(event)
    }
}
