//! `Std.Results` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/language/types/result-operations.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Results.*`.
///
/// **Documentation:** `docs/pascal/language/types/result-operations.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ResultIntrinsic {
    Unwrap = 90,
    UnwrapOr = 91,
    IsOk = 92,
    IsError = 93,
    /// `Std.Results.Map(R, F)` — `Ok(v)` → `Ok(F(v))`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/language/types/result-operations.md`
    Map = 130,
    /// `Std.Results.AndThen(R, F)` — `Ok(v)` → `F(v)`, `Error(e)` passthrough.
    ///
    /// **Documentation:** `docs/pascal/language/types/result-operations.md`
    AndThen = 131,
    /// `Std.Results.OrElse(R, F)` — `Ok(v)` passthrough, `Error(e)` → `F(e)`.
    ///
    /// **Documentation:** `docs/pascal/language/types/result-operations.md`
    OrElse = 132,
}
}
