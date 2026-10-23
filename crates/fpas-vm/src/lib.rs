//! Bytecode execution and source-level debugging for compiled FPAS programs.
//!
//! Execution and debugger termination return registered [`HostValue`] handles.
//! These preserve domain ownership after the session is dropped; inspection and
//! mutation remain guarded, while shutdown prohibits execution re-entry.
//! See `docs/pascal/program-structure/runtime-ownership.md`.

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "VM tests use unwrap/expect/panic to keep low-level bytecode assertions focused on behavior"
    )
)]

/// Canonical bounded serialization for detached diagnostic and trace values.
pub mod snapshot_value;

mod vm;
pub use vm::{DebugSnapshot, DebugTaskSnapshot, non_executing};

pub use fpas_std::ScreenSnapshot;
pub use vm::{
    BoundBreakpoint, BoundDataBreakpoint, BoundFunctionBreakpoint, CallbackSession, DataBreakpoint,
    DataBreakpointAccess, DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugBinaryOperation, DebugBreakpointLimits, DebugDataLocation, DebugDataLocationIdentity,
    DebugDataLocationKind, DebugDataLocationLifetime, DebugDictionaryMutationResult,
    DebugErrorKind, DebugEvaluateResult, DebugEvaluationCancelHandle, DebugEvaluationLimits,
    DebugExecutionLimits, DebugExpression, DebugForcedReturnResult, DebugFrame,
    DebugFrameRestartResult, DebugInspectionLimits, DebugPauseHandle, DebugRecordingEnvelope,
    DebugRecordingEvent, DebugRunResult, DebugScope, DebugScopeKind, DebugSession,
    DebugSessionError, DebugSessionState, DebugStop, DebugStopReason,
    DebugStorageInitializationResult, DebugStringMutationResult, DebugTask, DebugTaskEvent,
    DebugTaskEventKind, DebugTaskResultReplacement, DebugTaskState, DebugTerminalEvent,
    DebugTerminalHandle, DebugTerminalKeyEvent, DebugTerminalKeyKind, DebugTerminalMouseAction,
    DebugTerminalMouseButton, DebugTermination, DebugUnaryOperation, DebugVariable,
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
    DebuggeeChannelState, DebuggeeInputResult, Execution, FailureBinding, FailureDiagnostic,
    FailureFrame, FailureGlobal, FailureGlobals, FailureObservation, FailureScalar,
    FailureSnapshot, FailureTask, FailureTaskCatalog, FailureValue, FunctionBreakpoint,
    HostArgument, HostChild, HostEnumInfo, HostValue, HostValueError, LiveImageClassification,
    LiveImageReplaceResult, LiveImageUpdateClass, MAX_RECORDING_EVENTS, Paginated,
    RECORDING_ENVELOPE_VERSION, ShutdownHandle, SourceBreakpoint, SourceLocation, Vm, VmError,
    VmOutput,
};

pub use vm::{
    DEFAULT_TRACE_EVENTS, DebugTraceCaptureSource, DebugTraceEvent, DebugTraceIdentity,
    DebugTraceLocation, DebugTracePathStep, DebugTraceReport, DebugTraceWatch, MAX_TRACE_BYTES,
    MAX_TRACE_EVENTS,
};

pub use vm::{DebugScheduleReport, SCHEDULE_ALGORITHM};

pub use vm::{LineProfile, ProfileReport, ProfileSample, RoutineProfile};
