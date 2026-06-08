//! `Std.Test` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Test.*`.
///
/// **Documentation:** `docs/pascal/std/test.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TestIntrinsic {
    /// `Std.Test.AssertTrue(Cond)` — fail when `Cond` is false.
    AssertTrue = 348,
    /// `Std.Test.AssertFalse(Cond)` — fail when `Cond` is true.
    AssertFalse = 349,
    /// `Std.Test.AssertEquals(Expected, Actual)` for `integer` operands.
    AssertEqualsInteger = 350,
    /// `Std.Test.Fail(Msg)` — unconditional failure.
    Fail = 351,
    /// `Std.Test.Skip(Msg)` — mark test skipped (non-failing).
    Skip = 352,
    /// `Std.Test.AssertEquals(Expected, Actual)` for `boolean` operands.
    AssertEqualsBoolean = 353,
    /// `Std.Test.AssertEquals(Expected, Actual)` for `string` operands.
    AssertEqualsString = 354,
    /// `Std.Test.AssertEquals(Expected, Actual)` for `real` operands.
    AssertEqualsReal = 355,
}
