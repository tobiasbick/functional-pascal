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
    /// Wait for all tasks to complete. Pops `array of task`.
    ///
    /// **Documentation:** `docs/pascal/std/concurrency/task.md`
    WaitAll = 111,
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
}
}
