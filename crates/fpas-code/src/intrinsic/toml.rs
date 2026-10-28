//! `Std.Toml` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Toml.*`.
///
/// **Documentation:** `docs/pascal/std/text/toml.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TomlIntrinsic {
    /// `Std.Toml.Parse(Text)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/toml.md`
    Parse = 515,
    /// `Std.Toml.Stringify(Value)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/toml.md`
    Stringify = 516,
}
}
