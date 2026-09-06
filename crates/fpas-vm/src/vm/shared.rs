//! Shared runtime state types used by hosted services and task scheduling.

mod task_results;
mod timers;

pub(crate) use task_results::{TaskAnyPoll, TaskBatchPoll, TaskResultPoll, TaskResultState};
pub(crate) use timers::TaskTimers;
