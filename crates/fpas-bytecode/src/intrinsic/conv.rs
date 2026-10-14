//! `Std.Conv` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/conv.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Conv.*`.
///
/// **Documentation:** `docs/pascal/std/text/conv.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ConvIntrinsic {
    IntToStr = 40,
    RealToStr = 42,
    IntToReal = 45,
    /// `Std.Conv.BoolToStr(B)` — boolean to `'true'`/`'false'`.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    BoolToStr = 215,
    /// `Std.Conv.IntToHex(N)` — integer to hexadecimal string.
    ///
    /// **Documentation:** `docs/pascal/std/text/conv.md`
    IntToHex = 217,
}
}
