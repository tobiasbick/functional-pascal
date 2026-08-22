//! `Std.Task` symbol names and registry group.

std_symbol!(STD_TASK_WAIT = std_task!("Wait"));
std_symbol!(STD_TASK_WAIT_ALL = std_task!("WaitAll"));

pub(in crate::std_units) const STD_TASK_SYMBOLS: &[&str] = &[STD_TASK_WAIT, STD_TASK_WAIT_ALL];
