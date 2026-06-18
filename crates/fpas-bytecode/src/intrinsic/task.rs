//! `Std.Task` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root).

use num_enum::TryFromPrimitive;

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
}
