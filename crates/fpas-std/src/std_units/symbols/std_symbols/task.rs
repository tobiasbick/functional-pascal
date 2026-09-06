//! `Std.Task` symbol names and registry group.

/// Qualified name of the opaque `Std.Task.CancellationSource` record.
pub const STD_TASK_CANCELLATION_SOURCE: &str = std_task!("CancellationSource");
/// Qualified name of the opaque `Std.Task.CancellationToken` record.
pub const STD_TASK_CANCELLATION_TOKEN: &str = std_task!("CancellationToken");
std_symbol!(STD_TASK_CREATE_CANCELLATION_SOURCE = std_task!("CreateCancellationSource"));
std_symbol!(STD_TASK_GET_CANCELLATION_TOKEN = std_task!("GetCancellationToken"));
std_symbol!(STD_TASK_CANCEL = std_task!("Cancel"));
std_symbol!(STD_TASK_IS_CANCELLATION_REQUESTED = std_task!("IsCancellationRequested"));
std_symbol!(STD_TASK_WAIT = std_task!("Wait"));
std_symbol!(STD_TASK_WAIT_ALL = std_task!("WaitAll"));

pub(in crate::std_units) const STD_TASK_SYMBOLS: &[&str] = &[
    STD_TASK_CANCELLATION_SOURCE,
    STD_TASK_CANCELLATION_TOKEN,
    STD_TASK_CREATE_CANCELLATION_SOURCE,
    STD_TASK_GET_CANCELLATION_TOKEN,
    STD_TASK_CANCEL,
    STD_TASK_IS_CANCELLATION_REQUESTED,
    STD_TASK_WAIT,
    STD_TASK_WAIT_ALL,
];
