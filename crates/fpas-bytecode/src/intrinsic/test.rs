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

    /// `Std.Test.AssertScreenLine(Expected, Y)` — compare CRT row text.
    ///
    /// Stack: `Expected`, `Y` (`Y` on top). Reads the virtual screen directly.
    ///
    /// **Documentation:** `docs/pascal/std/test.md`, `docs/future/tui-tests-fpas/README.md`
    AssertScreenLine = 375,
    /// `Std.Test.AssertScreenCell(X, Y, Ch, Fg, Bg)` — compare one CRT cell.
    ///
    /// Stack: `X`, `Y`, `Ch`, `Fg`, `Bg` (`Bg` on top). Colors are packed CRT `0..=15`.
    ///
    /// **Documentation:** `docs/pascal/std/test.md`, `docs/future/tui-tests-fpas/README.md`
    AssertScreenCell = 376,
    /// `Std.Test.AssertViewRect(App, V, X, Y, W, H)` — compare a view rectangle.
    ///
    /// Stack: `App`, `V`, `X`, `Y`, `W`, `H` (`H` on top).
    ///
    /// **Documentation:** `docs/pascal/std/test.md`, `docs/future/tui-tests-fpas/README.md`
    AssertViewRect = 377,

    /// `Std.Test.PushReadLn(Line)` — queue one line for the next `Std.Console.ReadLn`.
    ///
    /// Stack: `Line`. Replaces pre-run `*.script.toml` readln events in native tests.
    ///
    /// **Documentation:** `docs/pascal/std/test.md`
    PushReadLn = 378,
}
