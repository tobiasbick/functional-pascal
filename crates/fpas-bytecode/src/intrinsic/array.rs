//! `Std.Arrays` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/collections/array/README.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Arrays.*`.
///
/// **Documentation:** `docs/pascal/std/collections/array/README.md`
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
    /// `Std.Arrays.Concat(A, B)` — concatenate two arrays.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    Concat = 234,
    /// `Std.Arrays.Fill(Value, Count)` — create array of Count copies of Value.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    Fill = 235,
    /// `Std.Arrays.Find(Arr, Pred)` — first element matching predicate, or None.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    Find = 236,
    /// `Std.Arrays.FindIndex(Arr, Pred)` — index of first match, or -1.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    FindIndex = 237,
    /// `Std.Arrays.Any(Arr, Pred)` — true if any element matches.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    Any = 238,
    /// `Std.Arrays.All(Arr, Pred)` — true if all elements match.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    All = 239,
    /// `Std.Arrays.FlatMap(Arr, F)` — map then flatten.
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    FlatMap = 240,
    /// `Std.Arrays.ForEach(Arr, F)` — apply F to each element (returns unit).
    ///
    /// **Documentation:** `docs/pascal/std/collections/array/README.md`
    ForEach = 241,
}
}
