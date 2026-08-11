//! Controlled source-level execution for debugger frontends.

mod breakpoints;
mod calls;
mod evaluation;
mod inspection;
mod mutation;
mod session;
mod tasks;
mod types;

#[cfg(test)]
mod tests;

pub use breakpoints::{BoundBreakpoint, SourceBreakpoint};
pub use evaluation::{
    DebugBinaryOperation, DebugEvaluateResult, DebugEvaluationLimits, DebugExpression,
    DebugUnaryOperation,
};
pub use inspection::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub use session::{DebugEvaluationCancelHandle, DebugPauseHandle, DebugSession};
pub use types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTermination, SourceLocation,
};
