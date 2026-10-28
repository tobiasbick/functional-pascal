//! `Std.Math.Random` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/numeric/random.md` (from the repository root).

use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Intrinsics for `Std.Math.Random.*`.
///
/// **Documentation:** `docs/pascal/std/numeric/random.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum RandomIntrinsic {
    /// `Std.Math.Random.Random` - random real in [0, 1).
    ///
    /// **Documentation:** `docs/pascal/std/numeric/random.md`
    Random = 231,
    /// `Std.Math.Random.RandomInt(Lo, Hi)` - random integer in [Lo, Hi].
    ///
    /// **Documentation:** `docs/pascal/std/numeric/random.md`
    RandomInt = 232,
    /// `Std.Math.Random.Randomize` - seed the RNG.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/random.md`
    Randomize = 233,
    /// `Std.Math.Random.SetSeed(Seed)` - replace the VM-local deterministic seed.
    ///
    /// **Documentation:** `docs/pascal/std/numeric/random.md`
    SetSeed = 610,
}
}
