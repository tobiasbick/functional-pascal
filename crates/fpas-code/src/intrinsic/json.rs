//! `Std.Encoding.Json` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/text/json.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Encoding.Json.*`.
///
/// **Documentation:** `docs/pascal/std/text/json.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum JsonIntrinsic {
    /// `Std.Encoding.Json.Parse(Text)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/json.md`
    Parse = 325,
    /// `Std.Encoding.Json.Stringify(Value)`.
    ///
    /// **Documentation:** `docs/pascal/std/text/json.md`
    Stringify = 326,
}
}
