//! Stable debugger session states, stops, results, and command errors.

use std::fmt;
use std::time::Duration;

/// Resource limits enforced by controlled debugger execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugExecutionLimits {
    /// Maximum instructions dispatched over the whole session.
    pub max_instructions: u64,
    /// Maximum wall-clock time for one resume operation.
    pub timeout: Duration,
    /// Maximum captured standard-output bytes over the whole session.
    pub max_output_bytes: usize,
    /// Maximum queued debuggee input bytes over the whole session.
    pub max_input_bytes: usize,
}

impl Default for DebugExecutionLimits {
    fn default() -> Self {
        Self {
            max_instructions: 100_000_000,
            timeout: Duration::from_secs(300),
            max_output_bytes: 1_048_576,
            max_input_bytes: 1_048_576,
        }
    }
}

/// One accepted debuggee input line queued for hosted `Read` / `ReadLn`.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebuggeeInputResult {
    /// UTF-8 bytes counted for this line, including the stored newline.
    pub bytes: usize,
    /// Cumulative accepted debuggee input bytes for the session.
    pub session_bytes: usize,
}

/// Source position resolved from an executable debugger sequence point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Portable source path.
    pub source: String,
    /// One-based source line.
    pub line: u32,
    /// One-based source column.
    pub column: u32,
}

/// Externally observable debugger session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSessionState {
    /// Execution is paused at a stable instruction boundary.
    Stopped,
    /// A resume operation is dispatching instructions.
    Running,
    /// Root execution completed or the session was disconnected.
    Terminated,
    /// Execution stopped because the program raised a runtime diagnostic.
    Failed,
}

/// Reason for a source-level stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStopReason {
    /// Initial launch stop before the entry sequence point executes.
    Entry,
    /// A verified source or function breakpoint was reached.
    Breakpoint,
    /// A verified global data breakpoint observed a matching store.
    DataBreakpoint,
    /// A cooperative pause request was observed.
    Pause,
    /// A step command reached its next source boundary.
    Step,
    /// Program execution raised a runtime diagnostic.
    RuntimeError,
}

/// Stable snapshot identifying one debugger stop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStop {
    /// Stop reason.
    pub reason: DebugStopReason,
    /// Runtime identity of the task responsible for this stop.
    pub task_id: u64,
    /// Source position when the stop has a debugger sequence point.
    pub location: Option<SourceLocation>,
    /// Global bytecode instruction address for deterministic protocol mapping.
    pub instruction: u32,
    /// Zero-based active call depth.
    pub call_depth: usize,
    /// First breakpoint identifier when the stop names one or more logical breakpoints.
    pub breakpoint_id: Option<u64>,
    /// All logical breakpoint identifiers bound to the reached sequence point.
    pub breakpoint_ids: Vec<u64>,
    /// Runtime diagnostic when `reason` is [`DebugStopReason::RuntimeError`].
    pub diagnostic: Option<fpas_diagnostics::Diagnostic>,
}

/// Stable lifecycle state for one FPAS task in a stopped debug session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTaskState {
    /// Ready to execute when the session resumes.
    Runnable,
    /// Currently executing one instruction inside the debugger driver.
    Running,
    /// Waiting for one or more retained task results.
    Waiting,
    /// Waiting for a task-local timer.
    Sleeping,
    /// Returned normally and no longer has an inspectable stack.
    Completed,
    /// Raised the runtime failure that stopped the session.
    Failed,
    /// Was cancelled by root termination, debugger disconnect, or `task.cancel`.
    Cancelled,
}

impl DebugTaskState {
    /// Return the stable lowercase protocol spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Sleeping => "sleeping",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Whether this state retains a stable worker snapshot for inspection.
    pub(super) const fn is_inspectable(self) -> bool {
        !matches!(self, Self::Completed | Self::Cancelled)
    }
}

/// Bounded protocol-neutral description of one FPAS task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugTask {
    /// Runtime task identity; the main task is `0`.
    pub id: u64,
    /// Stable human-readable task name for editor thread views.
    pub name: String,
    /// Lifecycle state captured at the current all-stop generation.
    pub state: DebugTaskState,
    /// Whether stack and frame inspection is valid at this stop.
    pub inspectable: bool,
    /// Whether session-wide continue and peer steps skip this task until resume.
    pub paused: bool,
}

/// Stable kind of task-lifecycle event emitted between debugger stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTaskEventKind {
    /// A spawned task became known to the debugger.
    Started,
    /// A spawned task completed or was cancelled.
    Exited,
}

/// Task-lifecycle change accumulated during one resume operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugTaskEvent {
    /// Runtime identity of the changed task.
    pub task_id: u64,
    /// Lifecycle transition exposed to protocol adapters.
    pub kind: DebugTaskEventKind,
}

/// Successful root termination information.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugTermination {
    /// Root return value.
    pub value: fpas_bytecode::Value,
    /// Number of dispatched packed instructions.
    pub instruction_count: u64,
}

/// Result of one continue or step operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugRunResult {
    /// Execution paused and remains inspectable.
    Stopped(DebugStop),
    /// Root execution completed.
    Terminated(DebugTermination),
}

/// Stable debugger command error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugErrorKind {
    /// The command is not permitted in the current session state.
    InvalidState,
    /// The selected task is unknown or has no inspectable stopped state.
    UnknownTask,
    /// A breakpoint identifier is unknown.
    UnknownBreakpoint,
    /// Breakpoint count, identity, or physical binding limits were exceeded.
    BreakpointLimit,
    /// A frame identifier belongs to an expired or different stop snapshot.
    UnknownFrame,
    /// Frame restart is not available for this stop, task, frame, or register state.
    FrameRestartUnsupported,
    /// An instruction-pointer change cannot be proven from existing bytecode dataflow.
    InstructionChangeUnsupported,
    /// Debugger-created tasks cannot be proven without executing a spawn.
    TaskCreateUnsupported,
    /// Restarting a task would invent a new runtime identity.
    TaskRestartUnsupported,
    /// Mixing process stdin onto the protocol stream remains unsupported.
    LiveInputUnsupported,
    /// Queued debuggee input would exceed the session input limit.
    DebuggeeInputLimit,
    /// The debuggee input channel is closed or has already signaled EOF.
    DebuggeeInputClosed,
    /// Forced return is not available for this stop, frame, task, or result category.
    FrameReturnUnsupported,
    /// A value-returning function was force-returned without an expression.
    FrameReturnValueRequired,
    /// A procedure was force-returned with an unexpected expression.
    FrameReturnValueUnexpected,
    /// The evaluated return value does not match the declared portable result type.
    FrameReturnType,
    /// A task does not own an available unconsumed completed result.
    TaskResultReplacementUnsupported,
    /// A completed task replacement does not match its declared portable result type.
    TaskResultReplacementType,
    /// The requested variant name is not present on the target wrapper type.
    VariantUnknown,
    /// Construction fields are missing, extra, unknown, or ASCII-case-duplicate.
    VariantFieldSet,
    /// Seeded empty-storage initialization targeted a root that already holds a value.
    StorageAlreadyInitialized,
    /// A variables reference belongs to an expired or different stop snapshot.
    UnknownVariablesReference,
    /// The requested writable child does not exist in the current container.
    VariableTargetUnknown,
    /// The requested writable reference belongs to a previous stop generation.
    VariableTargetExpired,
    /// The selected source root is immutable.
    VariableNotMutable,
    /// The selected synthetic or aggregate path is deliberately not assignable.
    VariablePathUnsupported,
    /// The selected source binding has no initialized live value.
    VariableUninitialized,
    /// The replacement value does not match the declared portable type.
    VariableValueType,
    /// Live mutation storage cannot be accessed safely.
    VariableUnavailable,
    /// A dictionary insertion or replacement selected an existing key.
    DictionaryKeyExists,
    /// A dictionary removal or key replacement selected no existing key.
    DictionaryKeyMissing,
    /// A dictionary key replacement supplied an equal old and new key.
    DictionaryKeyUnchanged,
    /// An array insertion or removal index is outside the permitted sequence range.
    SequenceIndexOutOfBounds,
    /// A string character replacement expression did not produce exactly one Unicode scalar.
    StringCharacterRequired,
    /// A string character replacement supplied the character already stored at the index.
    StringCharacterUnchanged,
    /// A requested inspection page exceeds configured limits.
    InspectionLimit,
    /// An expression references no visible binding with the requested name.
    UnknownName,
    /// An expression references a visible binding that has no initialized value.
    UninitializedValue,
    /// Runtime values do not support the requested read-only expression operation.
    EvaluationType,
    /// Runtime values have valid types but violate an expression operation domain.
    EvaluationDomain,
    /// An expression exceeded a configured parser, evaluator, traversal, or output limit.
    EvaluationLimit,
    /// A mutable or opaque runtime value cannot be read safely without blocking.
    UnavailableValue,
    /// No exact executable callable matches the requested name or member.
    UnknownCallable,
    /// More than one exact callable candidate matches the request.
    AmbiguousCallable,
    /// A controlled call supplied the wrong number of arguments.
    CallArity,
    /// A controlled call has a denied transitive effect.
    ForbiddenCallEffect,
    /// Controlled call count, depth, detached-value, or instruction bounds were exceeded.
    CallLimit,
    /// Controlled call execution exceeded its wall-clock deadline.
    CallTimeout,
    /// Controlled call execution was cooperatively cancelled.
    CallCancelled,
    /// The detached worker reported a runtime failure.
    CallRuntime,
    /// One resume operation exceeded its wall-clock deadline.
    ExecutionTimeout,
    /// The session exhausted its instruction budget.
    InstructionLimit,
    /// Captured program output exceeded its byte budget.
    OutputLimit,
    /// A recording envelope would name a host filesystem path.
    RecordingHostPath,
}

/// Actionable debugger command failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSessionError {
    /// Stable machine-facing category.
    pub kind: DebugErrorKind,
    /// Human-readable failure detail.
    pub message: String,
    /// Concrete recovery guidance.
    pub hint: String,
}

impl DebugSessionError {
    pub(super) fn invalid_state(command: &'static str, state: DebugSessionState) -> Self {
        Self {
            kind: DebugErrorKind::InvalidState,
            message: format!("debug command `{command}` is invalid while the session is {state:?}"),
            hint: "Wait for a stopped event or launch a new debug session.".to_string(),
        }
    }
}

impl fmt::Display for DebugSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}\n  help: {}", self.message, self.hint)
    }
}

impl std::error::Error for DebugSessionError {}
