//! Breakpoint policy, stopped execution, task control, reload and replay.

#[path = "../breakpoint_assign.rs"]
mod breakpoint_assign;
#[path = "../completed_result.rs"]
mod completed_result;
#[path = "../data_breakpoints.rs"]
mod data_breakpoints;
#[path = "../forced_return.rs"]
mod forced_return;
#[path = "../frame_restart.rs"]
mod frame_restart;
#[path = "../function_breakpoints.rs"]
mod function_breakpoints;
#[path = "../hot_reload.rs"]
mod hot_reload;
#[path = "../instruction_change.rs"]
mod instruction_change;
#[path = "../location.rs"]
mod location;
#[path = "../quiescence.rs"]
mod quiescence;
#[path = "../record_replay.rs"]
mod record_replay;
#[path = "../runtime_failure_filters.rs"]
mod runtime_failure_filters;
#[path = "../task_control.rs"]
mod task_control;
#[path = "../task_debugging.rs"]
mod task_debugging;
#[path = "../task_lifecycle.rs"]
mod task_lifecycle;
