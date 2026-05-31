//! `Std.Json` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/json.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Json.*`.
///
/// **Documentation:** `docs/pascal/std/json.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum JsonIntrinsic {
    /// `Std.Json.Parse(Text)`.
    ///
    /// **Documentation:** `docs/pascal/std/json.md`
    Parse = 325,
    /// `Std.Json.Stringify(Value)`.
    ///
    /// **Documentation:** `docs/pascal/std/json.md`
    Stringify = 326,
}
