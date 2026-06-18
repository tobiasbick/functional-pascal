//! `Std.Conv` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/conv.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Conv.*`.
///
/// **Documentation:** `docs/pascal/std/text/conv.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ConvIntrinsic {
    IntToStr = 40,
    StrToInt = 41,
    RealToStr = 42,
    StrToReal = 43,
    CharToStr = 44,
    IntToReal = 45,
    /// `Std.Conv.BoolToStr(B)` — boolean to `'true'`/`'false'`.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    BoolToStr = 215,
    /// `Std.Conv.StrToBool(S)` — parse `'true'`/`'false'` to boolean.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    StrToBool = 216,
    /// `Std.Conv.IntToHex(N)` — integer to hexadecimal string.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    IntToHex = 217,
    /// `Std.Conv.HexToInt(S)` — hexadecimal string to integer.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    HexToInt = 218,
}
