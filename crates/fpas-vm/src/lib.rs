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
    BoundBreakpoint, BoundFunctionBreakpoint, CallbackSession, DebugArrayMutationResult,
    DebugAssignmentSelector, DebugAssignmentTarget, DebugBinaryOperation, DebugBreakpointLimits,
    DebugDictionaryMutationResult, DebugErrorKind, DebugEvaluateResult,
    DebugEvaluationCancelHandle, DebugEvaluationLimits, DebugExecutionLimits, DebugExpression,
    DebugForcedReturnResult, DebugFrame, DebugFrameRestartResult, DebugInspectionLimits,
    DebugPauseHandle, DebugRunResult, DebugScope, DebugScopeKind, DebugSession, DebugSessionError,
    DebugSessionState, DebugStop, DebugStopReason, DebugStorageInitializationResult,
    DebugStringMutationResult, DebugTask, DebugTaskEvent, DebugTaskEventKind,
    DebugTaskResultReplacement, DebugTaskState, DebugTermination, DebugUnaryOperation,
    DebugVariable, DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField,
    DebugVariantInfo, DebuggeeChannelState, Execution, FunctionBreakpoint, Paginated,
    ShutdownHandle, SourceBreakpoint, SourceLocation, Vm, VmError, VmOutput,
};
