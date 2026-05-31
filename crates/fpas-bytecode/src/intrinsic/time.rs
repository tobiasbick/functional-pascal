//! `Std.Time` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/time.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Time.*`.
///
/// **Documentation:** `docs/pascal/std/time.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TimeIntrinsic {
    /// `Std.Time.TimestampMillis()` - UTC wall-clock milliseconds since the Unix epoch.
    ///
    /// **Documentation:** `docs/pascal/std/time.md`
    TimestampMillis = 321,
    /// `Std.Time.MonotonicMillis()` - monotonic milliseconds since runtime initialization.
    ///
    /// **Documentation:** `docs/pascal/std/time.md`
    MonotonicMillis = 322,
    /// `Std.Time.ElapsedMillis(Start)` - monotonic milliseconds since `Start`.
    ///
    /// **Documentation:** `docs/pascal/std/time.md`
    ElapsedMillis = 323,
    /// `Std.Time.Sleep(Milliseconds)` - block for a non-negative millisecond count.
    ///
    /// **Documentation:** `docs/pascal/std/time.md`
    Sleep = 324,
}
