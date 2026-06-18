//! `Std.Graph` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`, `docs/pascal/std/graph/app/README.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Graph.Application.*` routines.
///
/// **Documentation:** `docs/pascal/std/graph/session.md`, `docs/pascal/std/graph/app/README.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum GraphIntrinsic {
    /// `Std.Graph.Application.Open(Width, Height, Title)`.
    ApplicationOpen = 292,
    /// `Std.Graph.Application.Close(App)`.
    ApplicationClose = 293,
    /// `Std.Graph.Application.Size(App)`.
    ApplicationSize = 294,
    /// `Std.Graph.Application.RequestRedraw(App)`.
    ApplicationRequestRedraw = 295,
    /// `Std.Graph.Application.Configure(App, Handlers)`.
    ApplicationConfigure = 296,
    /// `Std.Graph.Application.UploadFrame(App, Width, Height, Pixels)`.
    ApplicationUploadFrame = 297,
    /// `Std.Graph.Application.Clear(App, Color)`.
    ApplicationClear = 298,
    /// `Std.Graph.Application.PutPixel(App, X, Y, Color)`.
    ApplicationPutPixel = 299,
    /// `Std.Graph.Application.Present(App)`.
    ApplicationPresent = 300,
    /// `Std.Graph.Application.DrawLine(...)`.
    ApplicationDrawLine = 301,
    /// `Std.Graph.Application.DrawRect(...)`.
    ApplicationDrawRect = 302,
    /// `Std.Graph.Application.FillRect(...)`.
    ApplicationFillRect = 303,
    /// `Std.Graph.Application.DrawCircle(...)`.
    ApplicationDrawCircle = 304,
    /// `Std.Graph.Application.DrawText(...)`.
    ApplicationDrawText = 305,
    /// Hosted application loop (`Std.Graph.Application.Run`).
    ApplicationRun = 331,
    /// Request cooperative quit during a hosted run.
    HostRequestQuit = 332,
    /// Register `function (Application, KeyEvent): boolean`.
    HostRegisterOnKeyPressed = 333,
    /// Register `procedure (Application, Size)`.
    HostRegisterOnResize = 334,
    /// Process at most one hosted event.
    HostProcessNext = 335,
    /// Register `procedure (Application)`.
    HostRegisterOnPaint = 336,
    /// Dispatch pending redraw through `OnPaint`.
    HostDispatchRedraw = 337,
    /// Register idle handler and interval.
    HostRegisterOnIdle = 338,
    /// Register `procedure (Application, ExitReason)`.
    HostRegisterOnExit = 339,
    /// Register `procedure (Application, Event)`.
    HostRegisterOnMouse = 340,
    /// Register `procedure (Application, Event)`.
    HostRegisterOnWheel = 341,
    /// Register `procedure (Application)`.
    HostRegisterOnCloseRequested = 342,

    /// Open a headless graph session for native FPAS tests (`Application.OpenForTest`).
    ///
    /// Stack: `Width`, `Height` (`Height` on top). Pushes `Application`.
    ///
    /// **Documentation:** `docs/pascal/std/graph/app/README.md`, `docs/pascal/std/testing/test.md`
    OpenForTest = 379,

    /// Enqueue one key event for the active hosted graph run (`Application.TestSendKey`).
    ///
    /// Stack: `Application`, `Std.Console.KeyEvent`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/graph/app/README.md`, `docs/pascal/std/testing/test.md`
    TestSendKey = 380,
}
