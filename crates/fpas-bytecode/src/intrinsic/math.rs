//! `Std.Math` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/numeric/math.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Math.*`.
///
/// **Documentation:** `docs/pascal/std/numeric/math.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum MathIntrinsic {
    Sqrt = 60,
    Pow = 61,
    Floor = 62,
    Ceil = 63,
    Round = 64,
    Sin = 65,
    Cos = 66,
    Log = 67,
    Min = 68,
    Max = 69,
    Abs = 70,
    /// `Std.Math.Tan(R)` — tangent.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Tan = 219,
    /// `Std.Math.ArcSin(R)` — arcsine.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    ArcSin = 220,
    /// `Std.Math.ArcCos(R)` — arccosine.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    ArcCos = 221,
    /// `Std.Math.ArcTan(R)` — arctangent.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    ArcTan = 222,
    /// `Std.Math.ArcTan2(Y, X)` — two-argument arctangent.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    ArcTan2 = 223,
    /// `Std.Math.Exp(R)` — e^R.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Exp = 224,
    /// `Std.Math.Log10(R)` — base-10 logarithm.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Log10 = 225,
    /// `Std.Math.Log2(R)` — base-2 logarithm.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Log2 = 226,
    /// `Std.Math.Trunc(R)` — truncate toward zero, return integer.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Trunc = 227,
    /// `Std.Math.Frac(R)` — fractional part.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Frac = 228,
    /// `Std.Math.Sign(X)` — sign (-1, 0, 1), polymorphic integer/real.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Sign = 229,
    /// `Std.Math.Clamp(X, Lo, Hi)` — clamp to range, polymorphic.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/math.md`
    Clamp = 230,
}
