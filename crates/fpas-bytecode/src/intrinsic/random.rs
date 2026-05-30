//! `Std.Random` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/random.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Random.*`.
///
/// **Documentation:** `docs/pascal/std/random.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum RandomIntrinsic {
    /// `Std.Random.Random` - random real in [0, 1).
    ///
    /// **Documentation:** `docs/pascal/std/random.md`
    Random = 231,
    /// `Std.Random.RandomInt(Lo, Hi)` - random integer in [Lo, Hi].
    ///
    /// **Documentation:** `docs/pascal/std/random.md`
    RandomInt = 232,
    /// `Std.Random.Randomize` - seed the RNG.
    ///
    /// **Documentation:** `docs/pascal/std/random.md`
    Randomize = 233,
}
