//! `Std.Task` symbol names and registry group.

/// Opaque group owner.
pub const STD_TASK_TASK_GROUP: &str = std_task!("TaskGroup");
/// Report of a grouped child's failure.
pub const STD_TASK_TASK_FAILURE: &str = std_task!("TaskFailure");
/// Distinct ordinary and runtime failure classes.
pub const STD_TASK_TASK_FAILURE_KIND: &str = std_task!("TaskFailureKind");

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
std_symbol!(STD_TASK_TRY_SEND = std_task!("TrySend"));
std_symbol!(STD_TASK_SEND_WITH_CANCELLATION = std_task!("SendWithCancellation"));
std_symbol!(STD_TASK_SEND_WITH_TIMEOUT = std_task!("SendWithTimeout"));
std_symbol!(STD_TASK_RECEIVE = std_task!("Receive"));
std_symbol!(STD_TASK_TRY_RECEIVE = std_task!("TryReceive"));
std_symbol!(STD_TASK_RECEIVE_WITH_CANCELLATION = std_task!("ReceiveWithCancellation"));
std_symbol!(STD_TASK_RECEIVE_WITH_TIMEOUT = std_task!("ReceiveWithTimeout"));
std_symbol!(STD_TASK_CLOSE_CHANNEL = std_task!("CloseChannel"));
std_symbol!(STD_TASK_WAIT = std_task!("Wait"));
/// Qualified name of the opaque single-use selection case.
pub const STD_TASK_WAIT_CASE: &str = std_task!("WaitCase");
std_symbol!(STD_TASK_RECEIVE_CASE = std_task!("ReceiveCase"));
std_symbol!(STD_TASK_SEND_CASE = std_task!("SendCase"));
std_symbol!(STD_TASK_TASK_CASE = std_task!("TaskCase"));
std_symbol!(STD_TASK_TIMER_CASE = std_task!("TimerCase"));
std_symbol!(STD_TASK_CANCELLATION_CASE = std_task!("CancellationCase"));
std_symbol!(STD_TASK_SELECT = std_task!("Select"));
std_symbol!(STD_TASK_CREATE_TASK_GROUP = std_task!("CreateTaskGroup"));
std_symbol!(STD_TASK_START_TASK_IN_GROUP = std_task!("StartTaskInGroup"));
std_symbol!(STD_TASK_START_SUPERVISED_TASK = std_task!("StartSupervisedTask"));
std_symbol!(STD_TASK_GET_TASK_GROUP_TOKEN = std_task!("GetTaskGroupToken"));
std_symbol!(STD_TASK_CANCEL_TASK_GROUP = std_task!("CancelTaskGroup"));
std_symbol!(STD_TASK_CLOSE_TASK_GROUP = std_task!("CloseTaskGroup"));
std_symbol!(STD_TASK_CLOSE_WAIT_CASE = std_task!("CloseWaitCase"));
std_symbol!(STD_TASK_WAIT_ALL = std_task!("WaitAll"));
std_symbol!(STD_TASK_WAIT_ANY = std_task!("WaitAny"));
std_symbol!(STD_TASK_WAIT_ANY_WITH_TIMEOUT = std_task!("WaitAnyWithTimeout"));
std_symbol!(STD_TASK_WAIT_ANY_WITH_CANCELLATION = std_task!("WaitAnyWithCancellation"));

pub(in crate::std_units) const STD_TASK_SYMBOLS: &[&str] = &[
    STD_TASK_CANCELLATION_SOURCE,
    STD_TASK_WAIT_CASE,
    STD_TASK_TASK_GROUP,
    STD_TASK_TASK_FAILURE,
    STD_TASK_TASK_FAILURE_KIND,
    STD_TASK_CREATE_TASK_GROUP,
    STD_TASK_START_TASK_IN_GROUP,
    STD_TASK_START_SUPERVISED_TASK,
    STD_TASK_GET_TASK_GROUP_TOKEN,
    STD_TASK_CANCEL_TASK_GROUP,
    STD_TASK_CLOSE_TASK_GROUP,
    STD_TASK_RECEIVE_CASE,
    STD_TASK_SEND_CASE,
    STD_TASK_TASK_CASE,
    STD_TASK_TIMER_CASE,
    STD_TASK_CANCELLATION_CASE,
    STD_TASK_SELECT,
    STD_TASK_CLOSE_WAIT_CASE,
    STD_TASK_CANCELLATION_TOKEN,
    STD_TASK_CREATE_CANCELLATION_SOURCE,
    STD_TASK_GET_CANCELLATION_TOKEN,
    STD_TASK_CANCEL,
    STD_TASK_IS_CANCELLATION_REQUESTED,
    STD_TASK_CREATE_CHANNEL,
    STD_TASK_SEND,
    STD_TASK_TRY_SEND,
    STD_TASK_SEND_WITH_CANCELLATION,
    STD_TASK_SEND_WITH_TIMEOUT,
    STD_TASK_RECEIVE,
    STD_TASK_TRY_RECEIVE,
    STD_TASK_RECEIVE_WITH_CANCELLATION,
    STD_TASK_RECEIVE_WITH_TIMEOUT,
    STD_TASK_CLOSE_CHANNEL,
    STD_TASK_WAIT,
    STD_TASK_WAIT_ALL,
    STD_TASK_WAIT_ANY,
    STD_TASK_WAIT_ANY_WITH_TIMEOUT,
    STD_TASK_WAIT_ANY_WITH_CANCELLATION,
];
