//! `Std.Text.Parse` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Text.Parse.*`.
///
/// **Documentation:** `docs/pascal/std/text/parse.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ParseIntrinsic {
    /// `Std.Text.Parse.Int(Text)` - parse Pascal integer text into `Result of (integer, string)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/parse.md`
    Int = 327,
    /// `Std.Text.Parse.Real(Text)` - parse Pascal real text into `Result of (real, string)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/parse.md`
    Real = 328,
    /// `Std.Text.Parse.Bool(Text)` - parse boolean text into `Result of (boolean, string)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/parse.md`
    Bool = 329,
    /// `Std.Text.Parse.HexInt(Text)` - parse explicit hexadecimal text into `Result of (integer, string)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/parse.md`
    HexInt = 218,
}
}
