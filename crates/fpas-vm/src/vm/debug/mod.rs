//! Controlled source-level execution for debugger frontends.

mod breakpoints;
mod inspection;
mod session;
mod tasks;
mod types;

#[cfg(test)]
mod tests;

pub use breakpoints::{BoundBreakpoint, SourceBreakpoint};
pub use inspection::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub use session::{DebugPauseHandle, DebugSession};
pub use types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTermination, SourceLocation,
};
