//! `Std.Task` symbol names and registry group.

/// Qualified name of the opaque `Std.Task.CancellationSource` record.
pub const STD_TASK_CANCELLATION_SOURCE: &str = std_task!("CancellationSource");
/// Qualified name of the opaque `Std.Task.CancellationToken` record.
pub const STD_TASK_CANCELLATION_TOKEN: &str = std_task!("CancellationToken");
std_symbol!(STD_TASK_CREATE_CANCELLATION_SOURCE = std_task!("CreateCancellationSource"));
std_symbol!(STD_TASK_GET_CANCELLATION_TOKEN = std_task!("GetCancellationToken"));
std_symbol!(STD_TASK_CANCEL = std_task!("Cancel"));
std_symbol!(STD_TASK_IS_CANCELLATION_REQUESTED = std_task!("IsCancellationRequested"));
std_symbol!(STD_TASK_CREATE_CHANNEL = std_task!("CreateChannel"));
std_symbol!(STD_TASK_SEND = std_task!("Send"));
std_symbol!(STD_TASK_SEND_WITH_CANCELLATION = std_task!("SendWithCancellation"));
std_symbol!(STD_TASK_RECEIVE = std_task!("Receive"));
std_symbol!(STD_TASK_RECEIVE_WITH_CANCELLATION = std_task!("ReceiveWithCancellation"));
std_symbol!(STD_TASK_CLOSE_CHANNEL = std_task!("CloseChannel"));
std_symbol!(STD_TASK_WAIT = std_task!("Wait"));
std_symbol!(STD_TASK_WAIT_ALL = std_task!("WaitAll"));

pub(in crate::std_units) const STD_TASK_SYMBOLS: &[&str] = &[
    STD_TASK_CANCELLATION_SOURCE,
    STD_TASK_CANCELLATION_TOKEN,
    STD_TASK_CREATE_CANCELLATION_SOURCE,
    STD_TASK_GET_CANCELLATION_TOKEN,
    STD_TASK_CANCEL,
    STD_TASK_IS_CANCELLATION_REQUESTED,
    STD_TASK_CREATE_CHANNEL,
    STD_TASK_SEND,
    STD_TASK_SEND_WITH_CANCELLATION,
    STD_TASK_RECEIVE,
    STD_TASK_RECEIVE_WITH_CANCELLATION,
    STD_TASK_CLOSE_CHANNEL,
    STD_TASK_WAIT,
    STD_TASK_WAIT_ALL,
];
