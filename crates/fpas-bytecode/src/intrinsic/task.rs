//! `Std.Task` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Task.*`.
///
/// **Documentation:** `docs/pascal/std/concurrency/task.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TaskIntrinsic {
    /// Wait for a task to complete. Pops `task`, pushes its return value.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    Wait = 110,
    /// CreateTaskGroup group ownership operation; see `docs/pascal/std/concurrency/task.md`.
    CreateTaskGroup = 558,
    /// StartTaskInGroup group ownership operation; see `docs/pascal/std/concurrency/task.md`.
    StartTaskInGroup = 559,
    /// GetTaskGroupToken group ownership operation; see `docs/pascal/std/concurrency/task.md`.
    GetTaskGroupToken = 560,
    /// CancelTaskGroup group ownership operation; see `docs/pascal/std/concurrency/task.md`.
    CancelTaskGroup = 561,
    /// CloseTaskGroup group ownership operation; see `docs/pascal/std/concurrency/task.md`.
    CloseTaskGroup = 562,
    /// Start a group-owned worker with bounded retries; see `docs/pascal/std/concurrency/task.md`.
    StartSupervisedTask = 563,
    /// Cooperatively close a group with a waiting budget; see `docs/pascal/std/concurrency/task.md`.
    CloseTaskGroupWithTimeout = 564,
    /// Close an already completed group without requesting cancellation.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    TryCloseCompletedTaskGroup = 565,
    /// ReceiveCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    ReceiveCase = 551,
    /// SendCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    SendCase = 552,
    /// TaskCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    TaskCase = 553,
    /// TimerCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    TimerCase = 554,
    /// CancellationCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    CancellationCase = 555,
    /// Select selection operation; see `docs/pascal/std/concurrency/task.md`.
    Select = 556,
    /// CloseWaitCase selection operation; see `docs/pascal/std/concurrency/task.md`.
    CloseWaitCase = 557,
    /// Wait for all tasks to complete. Pops `array of task`.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    WaitAll = 111,
    /// Return the first completed task's input index without consuming its result.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    WaitAny = 116,
    /// Wait for completion or a relative monotonic timeout.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    WaitAnyWithTimeout = 117,
    /// Wait for completion or cooperative cancellation.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    WaitAnyWithCancellation = 118,
    /// Create one VM-owned cancellation source.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    CreateCancellationSource = 112,
    /// Return a clonable token for a cancellation source.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    GetCancellationToken = 113,
    /// Request cancellation and report whether this call changed the state.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    Cancel = 114,
    /// Test whether cancellation was requested for a token.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    IsCancellationRequested = 115,
    /// Create one VM-owned bounded channel.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    CreateChannel = 537,
    /// Send one value, blocking while the channel is full.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    Send = 538,
    /// Try to send one value without blocking.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    TrySend = 543,
    /// Send one value until it succeeds, closes, or is cancelled.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    SendWithCancellation = 539,
    /// Send one value until it succeeds, closes, or the timeout expires.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    SendWithTimeout = 544,
    /// Receive one value, blocking while the channel is empty and open.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    Receive = 540,
    /// Try to receive one value without blocking.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    TryReceive = 545,
    /// Receive one value until it succeeds, closes, or is cancelled.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    ReceiveWithCancellation = 541,
    /// Receive one value until it succeeds, closes, or the timeout expires.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    ReceiveWithTimeout = 546,
    /// Close a channel and report whether this call changed its state.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    CloseChannel = 542,
}
}
