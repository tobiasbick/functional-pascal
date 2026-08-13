//! Controlled source-level execution for debugger frontends.

mod breakpoints;
mod calls;
mod evaluation;
mod forced_return;
mod inspection;
mod mutation;
mod session;
mod tasks;
mod types;

pub(in crate::vm::debug) use calls::construct_enum_payload;

#[cfg(test)]
mod tests;

pub use breakpoints::{BoundBreakpoint, SourceBreakpoint};
pub use evaluation::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
pub use forced_return::DebugForcedReturnResult;
pub use inspection::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub use mutation::{
    DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugDictionaryMutationResult, DebugStringMutationResult,
};
pub use session::{DebugEvaluationCancelHandle, DebugPauseHandle, DebugSession};
pub use types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTask, DebugTaskEvent, DebugTaskEventKind, DebugTaskState,
    DebugTermination, SourceLocation,
};
