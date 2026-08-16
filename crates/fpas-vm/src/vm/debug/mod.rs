//! Controlled source-level execution for debugger frontends.

mod breakpoints;
mod calls;
mod completed_result;
mod evaluation;
mod forced_return;
mod frame_restart;
mod inspection;
mod mutation;
mod routines;
mod session;
mod tasks;
mod types;

pub(in crate::vm::debug) use calls::{construct_enum, construct_enum_payload};

#[cfg(test)]
mod tests;

pub use breakpoints::{
    BoundBreakpoint, BoundFunctionBreakpoint, DebugBreakpointLimits, FunctionBreakpoint,
    SourceBreakpoint,
};
pub use completed_result::DebugTaskResultReplacement;
pub use evaluation::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
pub use forced_return::DebugForcedReturnResult;
pub use frame_restart::DebugFrameRestartResult;
pub use inspection::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub use mutation::{
    DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugDictionaryMutationResult, DebugStorageInitializationResult, DebugStringMutationResult,
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
};
pub use session::{DebugEvaluationCancelHandle, DebugPauseHandle, DebugSession};
pub use types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTask, DebugTaskEvent, DebugTaskEventKind, DebugTaskState,
    DebugTermination, SourceLocation,
};
