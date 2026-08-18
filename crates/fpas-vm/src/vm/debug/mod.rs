//! Controlled source-level execution for debugger frontends.

mod breakpoints;
mod calls;
mod completed_result;
mod evaluation;
mod forced_return;
mod frame_restart;
pub(in crate::vm) mod initializer_suppression;
mod inspection;
mod instruction;
mod io;
mod live_image;
mod location;
mod mutation;
mod recording;
mod routines;
mod session;
mod tasks;
mod types;

pub(in crate::vm::debug) use calls::{construct_enum, construct_enum_payload};

#[cfg(test)]
mod tests;

pub use breakpoints::{
    BoundBreakpoint, BoundDataBreakpoint, BoundFunctionBreakpoint, DataBreakpoint,
    DataBreakpointAccess, DebugBreakpointLimits, FunctionBreakpoint, SourceBreakpoint,
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
pub use io::DebuggeeChannelState;
pub use live_image::{LiveImageClassification, LiveImageUpdateClass};
pub use location::{
    DebugDataLocation, DebugDataLocationIdentity, DebugDataLocationKind, DebugDataLocationLifetime,
};
pub use mutation::{
    DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugDictionaryMutationResult, DebugStorageInitializationResult, DebugStringMutationResult,
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
};
pub use recording::{
    DebugRecordingEnvelope, DebugRecordingEvent, MAX_RECORDING_EVENTS, RECORDING_ENVELOPE_VERSION,
};
pub use session::{DebugEvaluationCancelHandle, DebugPauseHandle, DebugSession};
pub use types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTask, DebugTaskEvent, DebugTaskEventKind, DebugTaskState,
    DebugTermination, DebuggeeInputResult, SourceLocation,
};
