//! `Std.Dictionaries` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/language/types/dictionary-operations.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Dictionaries.*`.
///
/// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum DictIntrinsic {
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Length = 120,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    ContainsKey = 121,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Keys = 122,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Values = 123,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Remove = 124,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Get = 125,
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Merge = 126,
    /// `Std.Dictionaries.Map(D, F)` — transform every value; `F: function(V): V2`.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Map = 127,
    /// `Std.Dictionaries.Filter(D, F)` — keep entries where `F(K, V)` is true.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Filter = 128,
    /// Fold entries in insertion order from an explicit initial value.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionary-operations.md`
    Reduce = 572,
}
}
