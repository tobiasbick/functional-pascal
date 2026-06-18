//! `Std.Args` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/host/args.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Args.*`.
///
/// **Documentation:** `docs/pascal/std/host/args.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ArgsIntrinsic {
    /// `Std.Args.ParamCount()` - number of process arguments passed after the CLI separator.
    ///
    /// **Documentation:** `docs/pascal/std/host/args.md`
    ParamCount = 306,
    /// `Std.Args.ParamStr(Index)` - process argument by 0-based index.
    ///
    /// **Documentation:** `docs/pascal/std/host/args.md`
    ParamStr = 307,
}
