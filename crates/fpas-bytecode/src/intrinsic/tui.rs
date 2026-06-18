//! `Std.Tui` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use num_enum::TryFromPrimitive;

/// Intrinsics for `Std.Tui.*`.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum TuiIntrinsic {
    /// `Std.Tui.Application.Open()` — create/open a TUI application session.
    ///
    /// **Documentation:** `docs/pascal/std/tui/session.md`
    ApplicationOpen = 247,
    /// `Std.Tui.Application.Close(App)` — close a TUI application session.
    ///
    /// **Documentation:** `docs/pascal/std/tui/session.md`
    ApplicationClose = 248,
    /// `Std.Tui.Application.Size(App)` — current terminal size.
    ///
    /// **Documentation:** `docs/pascal/std/tui/session.md`
    ApplicationSize = 249,
    /// `Std.Tui.Application.RequestRedraw(App)` — mark the application as needing redraw.
    ///
    /// **Documentation:** `docs/pascal/std/tui/session.md`
    ApplicationRequestRedraw = 253,
    /// Register `function (Application, Std.Console.KeyEvent): boolean` for host key dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnKeyPressed = 256,
    /// Invoke the registered key handler; stack: `Application`, `Std.Console.KeyEvent` (key on top).
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostInvokeOnKeyPressed = 257,
    /// Register `procedure (Application, Std.Tui.Size)` for host resize dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnResize = 258,
    /// Poll the session, coalesce through `TuiHost`, dispatch at most one internal `UiEvent` to registered handlers.
    ///
    /// Stack: `Application`, `max_spins` (top). Pushes `integer` tag:
    /// `0` none, `1` key dispatched, `2` resize dispatched, `3` key without handler, `4` resize without handler.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostProcessNext = 259,
    /// Register `procedure (Application)` for host paint (`OnPaint`).
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnPaint = 260,
    /// If redraw is pending, consume it and invoke `OnPaint` when registered; otherwise no-op.
    ///
    /// Stack: `Application`. Pushes `integer`: `0` no redraw pending, `5` paint ran,
    /// `6` redraw pending but no handler (flag cleared).
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostDispatchRedraw = 261,
    /// Bounded host main loop: each iteration runs redraw dispatch then processes at most one input event.
    ///
    /// Stack: `Application`, `max_iterations` (`integer`, top, clamped to 1_000_000). Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRunLoop = 262,
    /// Request the bounded host run loop to stop after the current iteration.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRequestQuit = 263,
    /// Register `procedure (Application, Std.Tui.ExitReason)` for hosted `Run` / `OnExit`.
    ///
    /// Stack: `Application`, `OnExit` (function value, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnExit = 264,
    /// Hosted application loop (`Std.Tui.Application.Run`). Auto-requests first redraw, waits for quit.
    ///
    /// Stack: `Application`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    ApplicationRun = 265,
    /// Register `procedure (Application)` plus an idle interval in milliseconds for host idle dispatch.
    ///
    /// Zero or negative milliseconds disable idle callbacks until re-registered.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnIdle = 266,
    /// `Std.Tui.Application.Configure(App, Handlers)` — apply a hosted-dispatch handler bundle.
    ///
    /// Replaces the currently registered hosted handlers for the active application session.
    /// Stack: `Application`, `Std.Tui.ApplicationHandlers`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    ApplicationConfigure = 267,
    /// Register `procedure (Application, Std.Console.Event)` for host mouse-event dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnMouse = 268,
    /// Register `procedure (Application, Std.Console.Event)` for bracketed-paste dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnPaste = 269,
    /// Register `procedure (Application, Std.Console.Event)` for terminal focus-gained dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnFocusGained = 270,
    /// Register `procedure (Application, Std.Console.Event)` for terminal focus-lost dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnFocusLost = 271,
    /// Register `procedure (Application)` for host-managed focus-gained dispatch (Tab traversal).
    ///
    /// Fires when a view in the focus chain gains focus.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnActivate = 272,
    /// Register `procedure (Application)` for host-managed focus-lost dispatch (Tab traversal).
    ///
    /// Fires when a view in the focus chain loses focus.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnDeactivate = 273,
    /// Register `procedure (Application, integer)` for host-resolved command dispatch.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnCommand = 274,
    /// Bind a `Std.Console.KeyEvent` shortcut to a command id.
    ///
    /// Stack: `Application`, `Std.Console.KeyEvent`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostBindCommand = 275,
    /// Push an application-defined modal id onto the host modal stack.
    ///
    /// Stack: `Application`, `ModalId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostEnterModal = 276,
    /// Pop the active host modal frame, if any.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostLeaveModal = 277,
    /// Return the active modal stack depth.
    ///
    /// Stack: `Application`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryModalDepth = 278,
    /// Register a host-managed view and return its opaque handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height` (`integer`, top). Pushes `ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterView = 279,
    /// Remove a host-managed view by handle.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostUnregisterView = 280,
    /// Append a host-managed view to the focus chain.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostPushChildView = 281,
    /// Return the focused host-managed view handle, or `None` when no view is focused.
    ///
    /// Stack: `Application`. Pushes `Option of ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryFocusedViewId = 282,
    /// Attach a host-managed view handle to the active modal scope.
    ///
    /// Stack: `Application`, `ViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostAttachViewToActiveModal = 283,
    /// Update the bounding rectangle for a host-managed view.
    ///
    /// Stack: `Application`, `ViewId`, `X`, `Y`, `Width`, `Height` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostSetViewRect = 284,
    /// Re-parent a host-managed view.
    ///
    /// Stack: `Application`, `ViewId`, `Parent` (`Option of ViewId`, top; `None` detaches back to the root list).
    /// Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostSetViewParent = 285,
    /// Register a view-local paint handler.
    ///
    /// Stack: `Application`, `ViewId`, `OnViewPaint` (function value, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostRegisterOnViewPaint = 286,
    /// Show a modal dialog rooted at a host-managed view subtree.
    ///
    /// Stack: `Application`, `ModalId`, `RootViewId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    ApplicationShowModal = 287,
    /// Close the active modal dialog shown through `ApplicationShowModal`.
    ///
    /// Stack: `Application`. Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    ApplicationCloseModal = 288,
    /// Bind a shortcut that is only active while the specified host-managed view or one of its descendants has focus.
    ///
    /// Stack: `Application`, `ViewId`, `Key`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostBindCommandToView = 289,
    /// Bind a shortcut that is only active for the current modal frame.
    ///
    /// Stack: `Application`, `Key`, `CommandId` (`integer`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostBindCommandToActiveModal = 290,
    /// Register a root dialog view and show it as the active modal.
    ///
    /// Stack: `Application`, `ModalId`, `X`, `Y`, `Width`, `Height` (`integer`, top). Pushes the root `ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    ApplicationShowDialog = 291,
    /// Register a host-managed solid-fill widget view and return its opaque handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `FillColor`, `TextColor`, `FillChar`
    /// (`Option` values on top). Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostCreateSolidFillView = 343,
    /// Register a host-managed menu bar view from a Pascal item model and return its handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `Items`, `Style`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostCreateMenuBarView = 344,
    /// Replace the item model for an existing menu bar widget view.
    ///
    /// Stack: `Application`, `ViewId`, `Items` (`array of MenuBarItem`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostSetMenuBarItems = 345,
    /// Register a host-managed status bar view from a Pascal segment model and return its handle.
    ///
    /// Stack: `Application`, `X`, `Y`, `Width`, `Height`, `Segments`, `Style`. Pushes `integer`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostCreateStatusBarView = 346,
    /// Replace the segment model for an existing status bar widget view.
    ///
    /// Stack: `Application`, `ViewId`, `Segments` (`array of StatusBarSegment`, top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    HostSetStatusBarSegments = 347,

    /// Open a headless TUI session with a fixed virtual screen size for native tests.
    ///
    /// Stack: `Width`, `Height` (`integer`, top). Pushes `Application`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    OpenForTest = 356,
    /// Process one queued hosted event and settle the resulting redraw.
    ///
    /// Stack: `Application`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestPump = 357,
    /// Drain queued events and pending redraws until idle.
    ///
    /// Stack: `Application`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestPumpUntilIdle = 358,
    /// Close a headless test session and reset hosted TUI state.
    ///
    /// Stack: `Application`. Pushes `()`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    CloseForTest = 359,

    /// Enqueue a keyboard event for the next test pump.
    ///
    /// Stack: `Application`, `KeyEvent` (`KeyEvent` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestSendKey = 360,
    /// Enqueue a full `Std.Console.Event` (typically mouse) for the next test pump.
    ///
    /// Stack: `Application`, `Event` (`Event` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestSendMouse = 361,
    /// Enqueue a mouse `Move` at one-based `(X, Y)`.
    ///
    /// Stack: `Application`, `X`, `Y` (`Y` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestMoveMouse = 362,
    /// Enqueue mouse `Down` then `Up` at one-based `(X, Y)`.
    ///
    /// Stack: `Application`, `X`, `Y` (`Y` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestClickMouse = 363,
    /// Enqueue a terminal resize event.
    ///
    /// Stack: `Application`, `Width`, `Height` (`Height` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestResize = 364,
    /// Enqueue bracketed-paste text.
    ///
    /// Stack: `Application`, `Text` (`Text` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestPaste = 365,
    /// Enqueue focus gained (`true`) or focus lost (`false`).
    ///
    /// Stack: `Application`, `Gained` (`Gained` on top). Does not push a value.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    TestFocus = 366,

    /// Read the logical CRT screen size.
    ///
    /// Stack: `Application`. Pushes `Std.Tui.Size`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryScreenSize = 367,
    /// Read one screen row as a string (`Y` one-based).
    ///
    /// Stack: `Application`, `Y` (`Y` on top). Pushes `string`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryScreenLine = 368,
    /// Read one CRT cell (`X`/`Y` one-based) as `ScreenCell`.
    ///
    /// Stack: `Application`, `X`, `Y` (`Y` on top). Pushes `ScreenCell`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryScreenCell = 369,

    /// List root view handles in root-list order.
    ///
    /// Stack: `Application`. Pushes `array of ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryRootViews = 370,

    /// Read the absolute terminal rectangle of a registered view.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Pushes `Std.Tui.Rect`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryViewRect = 371,
    /// Read the parent view handle, or `None` for roots.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Pushes `Option of ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryViewParent = 372,
    /// Read direct child view handles in sibling order.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Pushes `array of ViewId`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryViewChildren = 373,

    /// Read menu bar widget hover and submenu state.
    ///
    /// Stack: `Application`, `ViewId` (`ViewId` on top). Pushes `MenuBarState`.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md`
    QueryMenuBarState = 374,
    // **375..=377** are `Std.Test` screen/view assertions; **378** is `Std.Test.PushReadLn` (see `TestIntrinsic`).
    // Note: **348..=355** are owned by `Std.Test` (`TestIntrinsic`).
}
