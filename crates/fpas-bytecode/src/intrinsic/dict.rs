//! `Std.Dict` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/collections/dict.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Dict.*`.
///
/// **Documentation:** `docs/pascal/std/collections/dict.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum DictIntrinsic {
    /// **Documentation:** `docs/future/advanced-types.md`
    Length = 120,
    /// **Documentation:** `docs/future/advanced-types.md`
    ContainsKey = 121,
    /// **Documentation:** `docs/future/advanced-types.md`
    Keys = 122,
    /// **Documentation:** `docs/future/advanced-types.md`
    Values = 123,
    /// **Documentation:** `docs/future/advanced-types.md`
    Remove = 124,
    /// **Documentation:** `docs/pascal/std/collections/dict.md`
    Get = 125,
    /// **Documentation:** `docs/pascal/std/collections/dict.md`
    Merge = 126,
    /// `Std.Dict.Map(D, F)` — transform every value; `F: function(V): V2`.
    ///
    /// **Documentation:** `docs/pascal/std/collections/dict.md`
    Map = 127,
    /// `Std.Dict.Filter(D, F)` — keep entries where `F(K, V)` is true.
    ///
    /// **Documentation:** `docs/pascal/std/collections/dict.md`
    Filter = 128,
}
