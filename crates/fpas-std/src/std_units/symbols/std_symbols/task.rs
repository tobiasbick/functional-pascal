//! `Std.Task` symbol names and registry group.

pub const STD_TASK_WAIT: &str = std_task!("Wait");
pub const STD_TASK_WAIT_ALL: &str = std_task!("WaitAll");

pub(in crate::std_units) const STD_TASK_SYMBOLS: &[&str] = &[STD_TASK_WAIT, STD_TASK_WAIT_ALL];
