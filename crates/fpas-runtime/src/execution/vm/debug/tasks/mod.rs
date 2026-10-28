//! Deterministic single-lane task execution for source-debug sessions.

mod driver;

pub(super) use driver::{
    CompletedResultTargetError, DebugDispatch, DebugSchedule, DebugTaskRuntime, TaskCancelError,
    TaskHoldError,
};
