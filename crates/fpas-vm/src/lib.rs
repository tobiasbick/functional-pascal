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
    BoundBreakpoint, CallbackSession, DebugErrorKind, DebugExecutionLimits, DebugFrame,
    DebugInspectionLimits, DebugPauseHandle, DebugRunResult, DebugScope, DebugScopeKind,
    DebugSession, DebugSessionError, DebugSessionState, DebugStop, DebugStopReason,
    DebugTermination, DebugVariable, Execution, Paginated, ShutdownHandle, SourceBreakpoint,
    SourceLocation, Vm, VmError, VmOutput,
};
