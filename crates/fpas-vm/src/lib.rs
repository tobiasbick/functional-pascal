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
    BoundBreakpoint, CallbackSession, DebugArrayMutationResult, DebugAssignmentSelector,
    DebugAssignmentTarget, DebugBinaryOperation, DebugDictionaryMutationResult, DebugErrorKind,
    DebugEvaluateResult, DebugEvaluationCancelHandle, DebugEvaluationLimits, DebugExecutionLimits,
    DebugExpression, DebugForcedReturnResult, DebugFrame, DebugInspectionLimits, DebugPauseHandle,
    DebugRunResult, DebugScope, DebugScopeKind, DebugSession, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugStringMutationResult, DebugTask, DebugTaskEvent,
    DebugTaskEventKind, DebugTaskState, DebugTermination, DebugUnaryOperation, DebugVariable,
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
    Execution, Paginated, ShutdownHandle, SourceBreakpoint, SourceLocation, Vm, VmError, VmOutput,
};
