//! `Std.Options` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/result/option.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Options.*`.
///
/// **Documentation:** `docs/pascal/std/result/option.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum OptionIntrinsic {
    Unwrap = 94,
    UnwrapOr = 95,
    IsSome = 96,
    IsNone = 97,
    /// `Std.Options.Map(O, F)` — `Some(v)` → `Some(F(v))`, `None` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result/option.md`
    Map = 133,
    /// `Std.Options.AndThen(O, F)` — `Some(v)` → `F(v)`, `None` passthrough.
    ///
    /// **Documentation:** `docs/pascal/std/result/option.md`
    AndThen = 134,
    /// `Std.Options.OrElse(O, F)` — `Some(v)` passthrough, `None` → `F()`.
    ///
    /// **Documentation:** `docs/pascal/std/result/option.md`
    OrElse = 135,
}
}
