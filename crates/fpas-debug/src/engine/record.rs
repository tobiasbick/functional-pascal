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
        execution: fpas_vm::DebugExecutionLimits,
        hot_reload: bool,
    },
    Evaluate(fpas_vm::DebugEvaluateResult),
    Tasks {
        tasks: Vec<fpas_vm::DebugTask>,
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
        frames: Vec<fpas_vm::DebugFrame>,
        total: usize,
        task_id: u64,
    },
    Scopes {
        scopes: Vec<fpas_vm::DebugScope>,
    },
    Variables {
        variables: Vec<fpas_vm::DebugVariable>,
        total: usize,
    },
    InputQueued {
        bytes: usize,
        session_bytes: usize,
    },
    Breakpoint(fpas_vm::BoundBreakpoint),
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
        breakpoints: Vec<fpas_vm::BoundFunctionBreakpoint>,
    },
    DataBreakpoints {
        breakpoints: Vec<fpas_vm::BoundDataBreakpoint>,
    },
    RuntimeFilters {
        filters: Vec<String>,
    },
    Dictionary(fpas_vm::DebugDictionaryMutationResult),
    Array(fpas_vm::DebugArrayMutationResult),
    StringCharacter(fpas_vm::DebugStringMutationResult),
    ForcedReturn(fpas_vm::DebugForcedReturnResult),
    FrameRestart(fpas_vm::DebugFrameRestartResult),
    Location(fpas_vm::DebugDataLocation),
    Recording {
        envelope: fpas_vm::DebugRecordingEnvelope,
        capturing: bool,
        events: Vec<fpas_vm::DebugRecordingEvent>,
        truncated: bool,
    },
    RecordingStarted {
        capturing: bool,
        truncated: bool,
        event_count: usize,
    },
    LiveImage {
        class: fpas_vm::LiveImageUpdateClass,
        accepted: bool,
        applied: bool,
        version: u64,
        rollback_available: bool,
    },
    VariantDescription {
        target: String,
        description: fpas_vm::DebugVariantDescription,
    },
    VariantConstruct(fpas_vm::DebugVariantConstructionResult),
    Storage(fpas_vm::DebugStorageInitializationResult),
    TaskResult(fpas_vm::DebugTaskResultReplacement),
}

/// Engine-owned debugger event.
#[derive(Debug, Clone)]
pub(crate) enum DebugEvent {
    Initialized,
    Stopped(fpas_vm::DebugStop),
    Task(fpas_vm::DebugTaskEvent),
    Output {
        category: &'static str,
        text: String,
        sequence: Option<usize>,
        breakpoint_id: Option<u64>,
        location: Option<fpas_vm::SourceLocation>,
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
    SourceBreakpoint(fpas_vm::BoundBreakpoint),
    FunctionBreakpoint(fpas_vm::BoundFunctionBreakpoint),
    DataBreakpoint(fpas_vm::BoundDataBreakpoint),
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
