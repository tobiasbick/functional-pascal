//! `Std.Graph` intrinsic discriminants.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Graph.Application.*` Phase 1 routines.
///
/// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum GraphIntrinsic {
    /// `Std.Graph.Application.Open(Width, Height, Title)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationOpen = 292,
    /// `Std.Graph.Application.Close(App)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationClose = 293,
    /// `Std.Graph.Application.Size(App)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationSize = 294,
    /// `Std.Graph.Application.PollEvent(App)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationPollEvent = 295,
    /// `Std.Graph.Application.ReadEventTimeout(App, Milliseconds)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationReadEventTimeout = 296,
    /// `Std.Graph.Application.UploadFrame(App, Width, Height, Pixels)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationUploadFrame = 297,
    /// `Std.Graph.Application.Clear(App, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationClear = 298,
    /// `Std.Graph.Application.PutPixel(App, X, Y, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationPutPixel = 299,
    /// `Std.Graph.Application.Present(App)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationPresent = 300,
    /// `Std.Graph.Application.DrawLine(App, X1, Y1, X2, Y2, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationDrawLine = 301,
    /// `Std.Graph.Application.DrawRect(App, X, Y, Width, Height, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationDrawRect = 302,
    /// `Std.Graph.Application.FillRect(App, X, Y, Width, Height, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationFillRect = 303,
    /// `Std.Graph.Application.DrawCircle(App, CenterX, CenterY, Radius, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationDrawCircle = 304,
    /// `Std.Graph.Application.DrawText(App, X, Y, Text, Color)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationDrawText = 305,
}
