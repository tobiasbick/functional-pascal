//! `Std.Env` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/env.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Env.*`.
///
/// **Documentation:** `docs/pascal/std/env.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum EnvIntrinsic {
    /// `Std.Env.Get(Name)` - process environment lookup.
    ///
    /// **Documentation:** `docs/pascal/std/env.md`
    Get = 308,
    /// `Std.Env.Exists(Name)` - process environment presence check.
    ///
    /// **Documentation:** `docs/pascal/std/env.md`
    Exists = 309,
}
