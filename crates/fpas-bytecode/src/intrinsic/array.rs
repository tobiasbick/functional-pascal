//! `Std.Array` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/collections/array.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Array.*`.
///
/// **Documentation:** `docs/pascal/std/collections/array.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum ArrayIntrinsic {
    Length = 80,
    Sort = 81,
    Reverse = 82,
    Contains = 83,
    IndexOf = 84,
    Slice = 85,
    Map = 86,
    Filter = 87,
    Reduce = 88,
    /// `Std.Array.Concat(A, B)` — concatenate two arrays.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    Concat = 234,
    /// `Std.Array.Fill(Value, Count)` — create array of Count copies of Value.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    Fill = 235,
    /// `Std.Array.Find(Arr, Pred)` — first element matching predicate, or None.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    Find = 236,
    /// `Std.Array.FindIndex(Arr, Pred)` — index of first match, or -1.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    FindIndex = 237,
    /// `Std.Array.Any(Arr, Pred)` — true if any element matches.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    Any = 238,
    /// `Std.Array.All(Arr, Pred)` — true if all elements match.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    All = 239,
    /// `Std.Array.FlatMap(Arr, F)` — map then flatten.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    FlatMap = 240,
    /// `Std.Array.ForEach(Arr, F)` — apply F to each element (returns unit).
    ///
    /// **Documentation:** `docs/pascal/std/collections/array.md`
    ForEach = 241,
}
