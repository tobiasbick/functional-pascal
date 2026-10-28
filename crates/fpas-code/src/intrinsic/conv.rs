//! Scalar formatting, numeric conversion and native-error intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/format.md` and
//! `docs/pascal/std/numeric/math.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for text formatting, numeric conversion and native-error operations.
///
/// **Documentation:** `docs/pascal/std/text/format.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ConvIntrinsic {
    /// Automatic native-error text, including typed causes.
    /// Documentation: `docs/pascal/language/types/errors.md`.
    ErrorMessage = 700,
    /// Internal closed-pattern test for a statically checked generic error family.
    ErrorMatchesVariant = 701,
    IntToStr = 40,
    RealToStr = 42,
    /// `Std.Math.IntToReal(N)` — explicit integer-to-real conversion.
    IntToReal = 45,
    /// `Std.Text.Format.BoolToStr(B)` — boolean to `'true'`/`'false'`.
    ///
    /// **Documentation:** `docs/pascal/std/text/format.md`
    BoolToStr = 215,
    /// `Std.Text.Format.IntToHex(N)` — integer to hexadecimal string.
    ///
    /// **Documentation:** `docs/pascal/std/text/format.md`
    IntToHex = 217,
}
}
