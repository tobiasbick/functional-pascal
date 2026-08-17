#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        clippy::panic,
        reason = "VM tests use unwrap/expect/panic to keep low-level bytecode assertions focused on behavior"
    )
)]

mod vm;

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
    DebugTaskEventKind, DebugTaskResultReplacement, DebugTaskState, DebugTermination,
    DebugUnaryOperation, DebugVariable, DebugVariantConstructionResult, DebugVariantDescription,
    DebugVariantField, DebugVariantInfo, DebuggeeChannelState, DebuggeeInputResult, Execution,
    FunctionBreakpoint, Paginated, RECORDING_ENVELOPE_VERSION, ShutdownHandle, SourceBreakpoint,
    SourceLocation, Vm, VmError, VmOutput,
};
