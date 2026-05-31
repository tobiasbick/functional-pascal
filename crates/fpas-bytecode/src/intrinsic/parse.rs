//! `Std.Parse` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/parse.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Parse.*`.
///
/// **Documentation:** `docs/pascal/std/parse.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ParseIntrinsic {
    /// `Std.Parse.TryInt(Text)` - parse Pascal integer text into `Result of integer, string`.
    ///
    /// **Documentation:** `docs/pascal/std/parse.md`
    TryInt = 327,
    /// `Std.Parse.TryReal(Text)` - parse Pascal real text into `Result of real, string`.
    ///
    /// **Documentation:** `docs/pascal/std/parse.md`
    TryReal = 328,
    /// `Std.Parse.TryBool(Text)` - parse boolean text into `Result of boolean, string`.
    ///
    /// **Documentation:** `docs/pascal/std/parse.md`
    TryBool = 329,
}
