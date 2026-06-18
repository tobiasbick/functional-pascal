//! `Std.Result` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/result/result.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Result.*`.
///
/// **Documentation:** `docs/pascal/std/result/result.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ResultIntrinsic {
    Unwrap = 90,
    UnwrapOr = 91,
    IsOk = 92,
    IsError = 93,
    /// `Std.Result.Map(R, F)` — `Ok(v)` → `Ok(F(v))`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result/result.md`
    Map = 130,
    /// `Std.Result.AndThen(R, F)` — `Ok(v)` → `F(v)`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result/result.md`
    AndThen = 131,
    /// `Std.Result.OrElse(R, F)` — `Ok(v)` passthrough, `Error(e)` → `F(e)`.
    ///
    /// **Documentation:** `docs/pascal/std/result/result.md`
    OrElse = 132,
}
