//! `Std.Tui` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Tui.*`.
///
/// **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TuiIntrinsic {
    /// `Std.Tui.Application.Open()` — create/open a TUI application session.
    ///
    /// **Documentation:** `docs/pascal/std/tui.md`
    ApplicationOpen = 247,
    /// `Std.Tui.Application.Close(App)` — close a TUI application session.
    ///
    /// **Documentation:** `docs/pascal/std/tui.md`
    ApplicationClose = 248,
    /// `Std.Tui.Application.Size(App)` — current terminal size.
    ///
    /// **Documentation:** `docs/pascal/std/tui.md`
    ApplicationSize = 249,
    /// `Std.Tui.Application.RequestRedraw(App)` — mark the application as needing redraw.
    ///
    /// **Documentation:** `docs/pascal/std/tui.md`
    ApplicationRequestRedraw = 253,
    /// Register `function (Application, Std.Console.KeyEvent): boolean` for host key dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnKeyPressed = 256,
    /// Invoke the registered key handler; stack: `Application`, `Std.Console.KeyEvent` (key on top).
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostInvokeOnKeyPressed = 257,
    /// Register `procedure (Application, Std.Tui.Size)` for host resize dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnResize = 258,
    /// Poll the session, coalesce through `TuiHost`, dispatch at most one internal `UiEvent` to registered handlers.
    ///
    /// Stack: `Application`, `max_spins` (top). Pushes `integer` tag:
    /// `0` none, `1` key dispatched, `2` resize dispatched, `3` key without handler, `4` resize without handler.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostProcessNext = 259,
    /// Register `procedure (Application)` for host paint (`OnPaint`).
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnPaint = 260,
    /// If redraw is pending, consume it and invoke `OnPaint` when registered; otherwise no-op.
    ///
    /// Stack: `Application`. Pushes `integer`: `0` no redraw pending, `5` paint ran,
    /// `6` redraw pending but no handler (flag cleared).
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostDispatchRedraw = 261,
    /// Bounded host main loop: each iteration runs redraw dispatch then processes at most one input event.
    ///
    /// Stack: `Application`, `max_iterations` (`integer`, top, clamped to 1_000_000). Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRunLoop = 262,
    /// Request the bounded host run loop to stop after the current iteration.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRequestQuit = 263,
    /// Register `procedure (Application, Std.Tui.ExitReason)` for hosted `Run` / `OnExit`.
    ///
    /// Stack: `Application`, `OnExit` (function value, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnExit = 264,
    /// Hosted application loop (`Std.Tui.Application.Run`). Auto-requests first redraw, waits for quit.
    ///
    /// Stack: `Application`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    ApplicationRun = 265,
    /// Register `procedure (Application)` plus an idle interval in milliseconds for host idle dispatch.
    ///
    /// Zero or negative milliseconds disable idle callbacks until re-registered.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnIdle = 266,
    /// `Std.Tui.Application.Configure(App, Handlers)` — apply a hosted-dispatch handler bundle.
    ///
    /// Replaces the currently registered hosted handlers for the active application session.
    /// Stack: `Application`, `Std.Tui.ApplicationHandlers`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    ApplicationConfigure = 267,
    /// Register `procedure (Application, Std.Console.Event)` for host mouse-event dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnMouse = 268,
    /// Register `procedure (Application, Std.Console.Event)` for bracketed-paste dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnPaste = 269,
    /// Register `procedure (Application, Std.Console.Event)` for terminal focus-gained dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnFocusGained = 270,
    /// Register `procedure (Application, Std.Console.Event)` for terminal focus-lost dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnFocusLost = 271,
    /// Register `procedure (Application)` for host-managed focus-gained dispatch (Tab traversal).
    ///
    /// Fires when a view in the focus chain gains focus.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnActivate = 272,
    /// Register `procedure (Application)` for host-managed focus-lost dispatch (Tab traversal).
    ///
    /// Fires when a view in the focus chain loses focus.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnDeactivate = 273,
    /// Register `procedure (Application, integer)` for host-resolved command dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnCommand = 274,
    /// Bind a `Std.Console.KeyEvent` shortcut to a command id.
    ///
    /// Stack: `Application`, `Std.Console.KeyEvent`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostBindCommand = 275,
    /// Push an application-defined modal id onto the host modal stack.
    ///
    /// Stack: `Application`, `ModalId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostEnterModal = 276,
    /// Pop the active host modal frame, if any.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostLeaveModal = 277,
    /// Return the active host modal stack depth.
    ///
    /// Stack: `Application`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostModalDepth = 278,
    /// Register a host-managed view and return its opaque handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height` (`integer`, top). Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterView = 279,
    /// Remove a host-managed view by handle.
    ///
    /// Stack: `Application`, `ViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostUnregisterView = 280,
    /// Append a host-managed view to the focus chain.
    ///
    /// Stack: `Application`, `ViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostPushChildView = 281,
    /// Return the focused host-managed view handle, or `-1` when no view is focused.
    ///
    /// Stack: `Application`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostQueryFocusedViewId = 282,
    /// Attach a host-managed view handle to the active modal scope.
    ///
    /// Stack: `Application`, `ViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostAttachViewToActiveModal = 283,
    /// Update the bounding rectangle for a host-managed view.
    ///
    /// Stack: `Application`, `ViewId`, `X`, `Y`, `Width`, `Height` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostSetViewRect = 284,
    /// Re-parent a host-managed view.
    ///
    /// Stack: `Application`, `ViewId`, `ParentViewId` (`integer`, top; `-1` detaches back to the root list).
    /// Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostSetViewParent = 285,
    /// Register a view-local paint handler.
    ///
    /// Stack: `Application`, `ViewId`, `OnViewPaint` (function value, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostRegisterOnViewPaint = 286,
    /// Show a modal dialog rooted at a host-managed view subtree.
    ///
    /// Stack: `Application`, `ModalId`, `RootViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    ApplicationShowModal = 287,
    /// Close the active modal dialog shown through `ApplicationShowModal`.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    ApplicationCloseModal = 288,
    /// Bind a shortcut that is only active while the specified host-managed view or one of its descendants has focus.
    ///
    /// Stack: `Application`, `ViewId`, `Key`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostBindCommandToView = 289,
    /// Bind a shortcut that is only active for the current modal frame.
    ///
    /// Stack: `Application`, `Key`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostBindCommandToActiveModal = 290,
    /// Register a root dialog view and show it as the active modal.
    ///
    /// Stack: `Application`, `ModalId`, `X`, `Y`, `Width`, `Height` (`integer`, top). Pushes the root `ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    ApplicationShowDialog = 291,
    /// Register a host-managed solid-fill widget view and return its opaque handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `FillColor`, `TextColor`, `FillChar`
    /// (`Option` values on top). Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostCreateSolidFillView = 343,
    /// Register a host-managed menu bar view from a Pascal item model and return its handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `Items`, `Style`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostCreateMenuBarView = 344,
    /// Replace the item model for an existing menu bar widget view.
    ///
    /// Stack: `Application`, `ViewId`, `Items` (`array of MenuBarItem`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostSetMenuBarItems = 345,
    /// Register a host-managed status bar view from a Pascal segment model and return its handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `Segments`, `Style`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostCreateStatusBarView = 346,
    /// Replace the segment model for an existing status bar widget view.
    ///
    /// Stack: `Application`, `ViewId`, `Segments` (`array of StatusBarSegment`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui-app.md`
    HostSetStatusBarSegments = 347,
}
