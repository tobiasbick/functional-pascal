//! `Std.Proc` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Proc.*`.
///
/// **Documentation:** `docs/pascal/std/host/proc.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ProcIntrinsic {
    /// `Std.Proc.Run(Command, Args)` - run a host process and return its exit code.
    ///
    /// **Documentation:** `docs/pascal/std/host/proc.md`
    Run = 330,
}
