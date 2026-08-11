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
}

impl Default for DebugExecutionLimits {
    fn default() -> Self {
        Self {
            max_instructions: 100_000_000,
            timeout: Duration::from_secs(300),
            max_output_bytes: 1_048_576,
        }
    }
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
    /// A verified source breakpoint was reached.
    Breakpoint,
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
    /// Source position when the stop has a debugger sequence point.
    pub location: Option<SourceLocation>,
    /// Global bytecode instruction address for deterministic protocol mapping.
    pub instruction: u32,
    /// Zero-based active call depth.
    pub call_depth: usize,
    /// Breakpoint identifier when `reason` is [`DebugStopReason::Breakpoint`].
    pub breakpoint_id: Option<u64>,
    /// All logical breakpoint identifiers bound to the reached sequence point.
    pub breakpoint_ids: Vec<u64>,
    /// Runtime diagnostic when `reason` is [`DebugStopReason::RuntimeError`].
    pub diagnostic: Option<fpas_diagnostics::Diagnostic>,
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
    /// The executable can spawn tasks, which V1 debugging does not support.
    UnsupportedTasks,
    /// A breakpoint identifier is unknown.
    UnknownBreakpoint,
    /// A frame identifier belongs to an expired or different stop snapshot.
    UnknownFrame,
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
