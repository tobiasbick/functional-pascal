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
    /// `Std.Graph.Application.UploadFrame(App, Width, Height, Pixels)`.
    ///
    /// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`
    ApplicationUploadFrame = 296,
}
