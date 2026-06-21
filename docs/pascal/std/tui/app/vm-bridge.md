# VM bridge

## VM bridge (Phase 3–4)

These `[fpas_bytecode::Intrinsic](../../../../../crates/fpas-bytecode/src/intrinsic/mod.rs)` variants drive `fpas_std::TuiHost` from the VM. In Pascal they appear as **`Std.Tui.Application.Host*`** (see table below); stack order matches other TUI intrinsics: pass `Application`, duplicate with the bytecode `Dup` opcode when the handle is needed again.


| Intrinsic                     | Stack (bottom → top)                             | Result                                                                                                                                                                                                                                                                              |
| ----------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TuiHostRegisterOnKeyPressed` | `Application`, `function`                        | Registers `function (Application, Std.Console.KeyEvent): boolean` for invoke.                                                                                                                                                                                                       |
| `TuiHostInvokeOnKeyPressed`   | `Application`, `Std.Console.KeyEvent`            | Calls the registered function; pushes `boolean` (`consumed`).                                                                                                                                                                                                                       |
| `TuiHostRegisterOnResize`     | `Application`, `function`                        | Registers `procedure (Application, Std.Tui.Size)` (arity 2).                                                                                                                                                                                                                        |
| `TuiHostProcessNext`          | `Application`, `max_spins` (`integer`, top)      | Spins up to `max_spins` (clamped to `4096`, minimum one iteration) through the hosted input pump, then dispatches **at most one** supported hosted event. The host uses a typed internal outcome and encodes the temporary bridge integer only here: `0` no event, `1` key handler returned `true` (consumed), `2` resize dispatched, `3` key without handler, `4` resize without handler, `5` mouse dispatched, `7` mouse without handler, `8` paste dispatched, `9` paste without handler, `10` focus-gained dispatched, `11` focus-gained without handler, `12` focus-lost dispatched, `13` focus-lost without handler, `14` Tab traversal advanced, `15` Shift+Tab traversal retreated, `16` command dispatched, `17` command bound without handler, `18` key blocked by modal scope, `19` mouse blocked by modal scope, `20` command blocked by modal scope, `21` native widget consumed, `22` key handler returned `false`. Tags `14`/`15` require an eligible retained view; active modal scope restricts traversal to its views. |
| `TuiHostRegisterOnPaint`      | `Application`, `function`                        | Registers `procedure (Application)` (arity 1).                                                                                                                                                                                                                                      |
| `TuiHostRegisterOnIdle`       | `Application`, `integer`, `function`             | Registers `procedure (Application)` plus an idle interval in milliseconds. `Milliseconds <= 0` disables idle callbacks.                                                                                                                                                             |
| `TuiHostDispatchRedraw`       | `Application`                                    | If redraw is pending: runs registered `OnPaint` after `take_redraw_pending`, or clears the flag with tag `6` when no handler. Pushes `integer`: `0` not pending, `5` paint ran, `6` cleared without handler.                                                                        |
| `TuiHostRunLoop`              | `Application`, `max_iterations` (`integer`, top) | Bounded host loop: each iteration runs the same work as `TuiHostDispatchRedraw` then `TuiHostProcessNext` with a fixed inner `max_spins` of `64`. After each iteration, if `TuiHostRequestQuit` was observed, the loop stops and the quit flag is cleared. Otherwise stops when both steps would be idle (`0`). `max_iterations` is clamped to `0..=1_000_000`. Pushes `()`. |
| `TuiHostRequestQuit`          | `Application`                                    | Sets a flag read by `TuiHostRunLoop` after each iteration. Does not push a value.                                                                                                                                                                                                 |
| `TuiHostRegisterOnExit`       | `Application`, `function`                        | Registers `procedure (Application, ExitReason)` for the hosted `Application.Run` / `OnExit` path. The bounded `HostRunLoop` helper still does **not** invoke it.                                                                                                                  |
| `TuiHostRegisterOnMouse`      | `Application`, `function`                        | Registers `procedure (Application, Std.Console.Event)` (arity 2) for host mouse-event dispatch.                                                                                                                                                                                     |
| `TuiHostRegisterOnPaste`      | `Application`, `function`                        | Registers `procedure (Application, Std.Console.Event)` (arity 2) for bracketed-paste dispatch. Best-effort; only fires on terminals that report paste events (requires `Std.Console.EnablePaste`).                                                                                  |
| `TuiHostRegisterOnFocusGained` | `Application`, `function`                       | Registers `procedure (Application, Std.Console.Event)` (arity 2) for terminal focus-gained dispatch. Best-effort / optional.                                                                                                                                                       |
| `TuiHostRegisterOnFocusLost`  | `Application`, `function`                        | Registers `procedure (Application, Std.Console.Event)` (arity 2) for terminal focus-lost dispatch. Best-effort / optional.                                                                                                                                                         |
| `TuiHostRegisterOnActivate`   | `Application`, `function`                        | Registers `procedure (Application)` (arity 1) for host-managed focus-gained dispatch. Fires when traversal or pointer-down selects another retained view. |
| `TuiHostRegisterOnDeactivate` | `Application`, `function`                        | Registers `procedure (Application)` (arity 1) for host-managed focus-lost dispatch before `OnActivate` when a previous focused leaf existed. |
| `TuiHostRegisterOnCommand`    | `Application`, `function`                        | Registers `procedure (Application, integer)` (arity 2) for host-resolved command dispatch. The integer argument is the command id bound to the shortcut.                                                                                                                           |
| `TuiHostBindCommand`          | `Application`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event (kind, character, and modifier flags) to a command id. Rebinding the same key replaces the previous command.                                                                                                                                            |
| `TuiHostBindCommandToView`    | `Application`, `integer`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event to a command id for one host-managed view. The binding is eligible when that view or one of its descendants currently has focus.                                                                                                                        |
| `TuiHostBindCommandToActiveModal` | `Application`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event to a command id for the active modal frame only. The binding disappears when that modal frame is closed.                                                                                                                                             |
| `TuiHostEnterModal`           | `Application`, `integer`                         | Pushes an application-defined modal id onto the host modal stack. Does not push a value.                                                                                                                                                                                           |
| `TuiHostLeaveModal`           | `Application`                                    | Pops the active host modal frame, if any. Leaving an empty modal stack is a no-op. Does not push a value.                                                                                                                                                                          |
| `TuiHostSetActiveModalResult` | `Application`, `integer`                         | Sets the active modal result. Accepted codes are `1` for Accept, `2` for Cancel, and application-defined command results `>= 1000`; other codes or an empty modal stack are runtime errors. Does not push a value. |
| `TuiHostRegisterView`         | `Application`, `integer`, `integer`, `integer`, `integer` | Registers a host-managed view from `x`, `y`, `width`, `height` and pushes `Std.Tui.ViewId`. Registration order remains the host paint order.                                                                                                                        |
| `TuiHostUnregisterView`       | `Application`, `ViewId`                         | Removes a host-managed view by handle. Unknown handles are ignored. Does not push a value.                                                                                                                                                                                        |
| `TuiHostPushChildView`        | `Application`, `ViewId`                         | Compatibility adapter that marks a view selectable and a Tab stop. Traversal is derived from retained tree order and view state; there is no separate focus list. Does not push a value.                                                                                                                                                |
| `TuiQueryModalDepth`          | `Application`                                    | Pushes `integer`: the active modal stack depth.                                                                                                                                                                                                                                    |
| `TuiQueryFocusedViewId`       | `Application`                                    | Pushes `Option of ViewId`: the currently focused view handle, or `None` when no host-managed view is focused.                                                                                                                                                                               |
| `TuiQueryViewState`           | `Application`, `ViewId`                          | Pushes resolved `ViewState` flags for one live retained view. |
| `TuiQueryViewOptions`         | `Application`, `ViewId`                          | Pushes retained `ViewOptions` for one live view. |
| `TuiQueryResolvedView`        | `Application`, `ViewId`                          | Pushes absolute geometry, effective clip, state, and options as `ResolvedView`. |
| `TuiQueryViewKind`            | `Application`, `ViewId`                          | Pushes `ViewKind`; views without native widgets return `Generic`. |
| `TuiQuerySceneGraph`          | `Application`                                    | Pushes a consistent `array of ViewSnapshot` in back-to-front depth-first paint order, including hidden and clipped nodes. |
| `TuiHostAttachViewToActiveModal` | `Application`, `ViewId`                      | Attaches a host-managed view handle to the currently active modal frame. Attached views define the modal focus/mouse scope for the topmost modal. Does not push a value.                                                                                                      |
| `TuiHostSetViewRect`          | `Application`, `ViewId`, `integer`, `integer`, `integer`, `integer` | Updates a host-managed view handle to `x`, `y`, `width`, `height`. Unknown handles are ignored. Does not push a value.                                                                                                                                      |
| `TuiHostSetViewParent`        | `Application`, `ViewId`, `Option of ViewId`           | Reparents a host-managed view under `Parent`. Pass `None` to detach the view back to the root list. The view keeps its current absolute terminal rectangle during the reparenting step. Unknown handles are ignored. Does not push a value.                          |
| `TuiHostSetViewVisible`       | `Application`, `ViewId`, `boolean`                    | Updates retained visibility. Descendants resolve as hidden when an ancestor is hidden. Unknown handles are ignored. Does not push a value. |
| `TuiHostSetViewEnabled`       | `Application`, `ViewId`, `boolean`                    | Updates input/focus eligibility for one view and clears focus when disabling the focused view. Unknown handles are ignored. Does not push a value. |
| `TuiHostRegisterOnViewPaint`  | `Application`, `ViewId`, `function`          | Registers `procedure (Application, ViewId, Std.Tui.Rect)` (arity 3) as a view-local paint handler for one host-managed view. During hosted redraw, the host invokes it in tree paint order when that view intersects the current damage region.                              |
| `TuiApplicationConfigure`     | `Application`, `ApplicationHandlers`             | Applies a bundled hosted-dispatch configuration. Replaces the current hosted handlers with the record fields from `ApplicationHandlers`; `OnPaint` is required, optional handlers use `Some(Handler)` or `None`, and `OnIdleMilliseconds <= 0` disables idle callbacks.        |
| `TuiApplicationRun`           | `Application`                                    | Hosted loop entrypoint. Requires a previously registered global `OnPaint` handler, at least one local view paint handler, **or** at least one host widget view (`HostCreateSolidFillView`, `HostCreateMenuBarView`, or `HostCreateStatusBarView`), auto-requests the first redraw, blocks until `Application.HostRequestQuit(App)` is observed **or** the host stops the active run, records `ExitReason.UserQuit`, `ExitReason.HostStop`, `ExitReason.HostAndUserStop`, or `ExitReason.HostShutdown`, invokes `OnExit` when registered, and performs `Application.Close` semantics before returning. Pushes `()`. |
| `TuiApplicationShowModal`     | `Application`, `integer`, `ViewId`             | Pushes a modal frame anchored to the given root view. The root view is raised, the modal scope becomes that view subtree (plus any explicitly attached extra views), and focus is moved into that scope when possible. Does not push a value.                               |
| `TuiApplicationShowDialog`    | `Application`, `integer`, `integer`, `integer`, `integer`, `integer` | Registers a new root host view for `x`, `y`, `width`, `height`, shows it as the active modal dialog, and pushes the new root `ViewId`. Closing that modal automatically unregisters the owned root subtree.                                                     |
| `TuiApplicationCloseModal`    | `Application`                                    | Pops the active modal frame created by `Application.ShowModal`, `Application.ShowDialog`, or `Application.HostEnterModal`. Leaving an empty modal stack is a no-op. Does not push a value.                                                                                     |
| `TuiHostCreateSolidFillView`  | `Application`, `integer`, `integer`, `integer`, `integer`, `integer`, `Option of integer`, `Option of string` | Registers a host-managed solid-fill widget view from `x`, `y`, `width`, `height`, `FillColor`, optional `TextColor`, and optional `FillChar`. Pushes `ViewId`. |
| `TuiHostCreateMenuBarView`    | `Application`, `integer`, `integer`, `integer`, `integer`, `array of MenuBarItem`, `MenuBarStyle` | Registers a host-managed menu bar widget from geometry and a declarative item model. Pushes `ViewId`. |
| `TuiHostSetMenuBarItems`      | `Application`, `ViewId`, `array of MenuBarItem` | Replaces the menu bar item model for an existing menu bar widget `ViewId`. Does not push a value. |
| `TuiHostCreateStatusBarView`  | `Application`, `integer`, `integer`, `integer`, `integer`, `array of StatusBarSegment`, `StatusBarStyle` | Registers a host-managed status bar widget from geometry and a declarative segment model. Pushes `ViewId`. |
| `TuiHostSetStatusBarSegments` | `Application`, `ViewId`, `array of StatusBarSegment` | Replaces the status bar segment model for an existing status bar widget `ViewId`. Does not push a value. |
| `TuiHostCreateLabelView` through `TuiHostCreateRadioGroupView` | `Application`, geometry, control model | Creates one native retained control with default dialog styling. Pushes `ViewId`. |
| `TuiHostSetInputLineText`, `TuiHostSetCheckBoxChecked`, `TuiHostSetRadioGroupSelected` | `Application`, `ViewId`, value | Updates retained control model state and requests redraw. |
| `TuiQueryInputLineState`, `TuiQueryCheckBoxState`, `TuiQueryRadioGroupState` | `Application`, `ViewId` | Pushes the matching typed control-state record. |
| `TuiHostCreateListBoxView`, `TuiHostSetListBoxItems`, `TuiHostSetListBoxSelected`, `TuiQueryListBoxState` | `Application`, geometry/`ViewId`, list model | Creates and controls a wheel/key-scrolling retained list box. |

### Pascal names (registry + compiler)

| Pascal `Std.Tui` call | Maps to intrinsic |
| ----------------------- | ----------------- |
| `Application.HostRegisterOnKeyPressed(App, OnKeyPressed)` | `TuiHostRegisterOnKeyPressed` |
| `Application.HostInvokeOnKeyPressed(App, Key)` | `TuiHostInvokeOnKeyPressed` |
| `Application.HostRegisterOnResize(App, OnResize)` | `TuiHostRegisterOnResize` |
| `Application.HostProcessNext(App, MaxSpins)` | `TuiHostProcessNext` |
| `Application.HostRegisterOnPaint(App, OnPaint)` | `TuiHostRegisterOnPaint` |
| `Application.HostRegisterOnIdle(App, Milliseconds, OnIdle)` | `TuiHostRegisterOnIdle` |
| `Application.HostDispatchRedraw(App)` | `TuiHostDispatchRedraw` |
| `Application.HostRunLoop(App, MaxIterations)` | `TuiHostRunLoop` |
| `Application.HostRequestQuit(App)` | `TuiHostRequestQuit` |
| `Application.HostRegisterOnExit(App, OnExit)` | `TuiHostRegisterOnExit` |
| `Application.HostRegisterOnMouse(App, OnMouse)` | `TuiHostRegisterOnMouse` |
| `Application.HostRegisterOnPaste(App, OnPaste)` | `TuiHostRegisterOnPaste` |
| `Application.HostRegisterOnFocusGained(App, OnFocusGained)` | `TuiHostRegisterOnFocusGained` |
| `Application.HostRegisterOnFocusLost(App, OnFocusLost)` | `TuiHostRegisterOnFocusLost` |
| `Application.HostRegisterOnActivate(App, OnActivate)` | `TuiHostRegisterOnActivate` |
| `Application.HostRegisterOnDeactivate(App, OnDeactivate)` | `TuiHostRegisterOnDeactivate` |
| `Application.HostRegisterOnCommand(App, OnCommand)` | `TuiHostRegisterOnCommand` |
| `Application.HostBindCommand(App, Key, CommandId)` | `TuiHostBindCommand` |
| `Application.HostBindCommandToView(App, ViewId, Key, CommandId)` | `TuiHostBindCommandToView` |
| `Application.HostBindCommandToActiveModal(App, Key, CommandId)` | `TuiHostBindCommandToActiveModal` |
| `Application.HostEnterModal(App, ModalId)` | `TuiHostEnterModal` |
| `Application.HostLeaveModal(App)` | `TuiHostLeaveModal` |
| `Application.HostSetActiveModalResult(App, ResultCode)` | `TuiHostSetActiveModalResult` |
| `Application.QueryModalDepth(App)` | `TuiQueryModalDepth` |
| `Application.HostRegisterView(App, X, Y, Width, Height)` | `TuiHostRegisterView` |
| `Application.HostUnregisterView(App, ViewId)` | `TuiHostUnregisterView` |
| `Application.HostPushChildView(App, ViewId)` | `TuiHostPushChildView` |
| `Application.QueryFocusedViewId(App)` | `TuiQueryFocusedViewId` |
| `Application.QueryViewState(App, ViewId)` | `TuiQueryViewState` |
| `Application.QueryViewOptions(App, ViewId)` | `TuiQueryViewOptions` |
| `Application.QueryResolvedView(App, ViewId)` | `TuiQueryResolvedView` |
| `Application.QueryViewKind(App, ViewId)` | `TuiQueryViewKind` |
| `Application.QuerySceneGraph(App)` | `TuiQuerySceneGraph` |
| `Application.HostAttachViewToActiveModal(App, ViewId)` | `TuiHostAttachViewToActiveModal` |
| `Application.HostSetViewRect(App, ViewId, X, Y, Width, Height)` | `TuiHostSetViewRect` |
| `Application.HostSetViewParent(App, ViewId, Parent)` | `TuiHostSetViewParent` |
| `Application.HostSetViewVisible(App, ViewId, Visible)` | `TuiHostSetViewVisible` |
| `Application.HostSetViewEnabled(App, ViewId, Enabled)` | `TuiHostSetViewEnabled` |
| `Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)` | `TuiHostRegisterOnViewPaint` |
| `Application.Configure(App, Handlers)` | `TuiApplicationConfigure` |
| `Application.Run(App)` | `TuiApplicationRun` |
| `Application.ShowModal(App, ModalId, RootViewId)` | `TuiApplicationShowModal` |
| `Application.ShowDialog(App, ModalId, X, Y, Width, Height)` | `TuiApplicationShowDialog` |
| `Application.CloseModal(App)` | `TuiApplicationCloseModal` |
| `Application.HostCreateSolidFillView(App, X, Y, Width, Height, FillColor, TextColor, FillChar)` | `TuiHostCreateSolidFillView` |
| `Application.HostCreateMenuBarView(App, X, Y, Width, Height, Items, Style)` | `TuiHostCreateMenuBarView` |
| `Application.HostSetMenuBarItems(App, ViewId, Items)` | `TuiHostSetMenuBarItems` |
| `Application.HostCreateStatusBarView(App, X, Y, Width, Height, Segments, Style)` | `TuiHostCreateStatusBarView` |
| `Application.HostSetStatusBarSegments(App, ViewId, Segments)` | `TuiHostSetStatusBarSegments` |
| `Application.HostCreateLabelView` through `Application.HostCreateRadioGroupView` | `TuiHostCreateLabelView` through `TuiHostCreateRadioGroupView` |
| `Application.HostSetInputLineText`, `HostSetCheckBoxChecked`, `HostSetRadioGroupSelected` | Matching `TuiHostSet*` intrinsic |
| `Application.QueryInputLineState`, `QueryCheckBoxState`, `QueryRadioGroupState`, `QueryListBoxState` | Matching `TuiQuery*State` intrinsic |

Samples: [`examples/pascal/tui/host_dispatch_minimal.fpas`](../../../../../examples/pascal/tui/host_dispatch_minimal.fpas) (one `HostProcessNext` step), [`examples/pascal/tui/host_dispatch_paint.fpas`](../../../../../examples/pascal/tui/host_dispatch_paint.fpas) (register `OnPaint` + `HostDispatchRedraw`), [`examples/pascal/tui/host_dispatch_quit.fpas`](../../../../../examples/pascal/tui/host_dispatch_quit.fpas) (`HostRequestQuit` from `OnPaint` + `HostRunLoop`), [`examples/pascal/tui/show_dialog.fpas`](../../../../../examples/pascal/tui/show_dialog.fpas) (owned modal dialog with `HostSetActiveModalResult`), [`apps/ide/ide.fpasprj`](../../../../../apps/ide/ide.fpasprj) (menu bar + About dialog).

**Bytecode discriminants** (authoritative enum: [`TuiIntrinsic`](../../../../../crates/fpas-bytecode/src/intrinsic/tui.rs)): existing host and test APIs use **256..=388** as listed above; retained controls use **389..=399**, and list-box operations use **400..=403**. Native headless testing uses **356..=374** (see [Native TUI testing API](testing.md)). **348..=355** and **375..=378** are `Std.Test` intrinsics, not TUI.

`Application.Close` clears registered host handlers (`OnKeyPressed`, `OnResize`, `OnPaint`, `OnIdle`, `OnExit`, `OnMouse`, `OnPaste`, `OnFocusGained`, `OnFocusLost`, `OnActivate`, `OnDeactivate`, `OnCommand`), clears local view paint handlers and command maps, resets the host pump, clears the view registry (focus path and pointer capture included), clears global command bindings, clears the modal stack, and closes the session.

### Modal host state

See [Modals and dialogs](modals.md) for the full guide. Summary:

`Application.ShowModal(App, ModalId, RootViewId)` pushes an application-defined modal id together with a root host view, raises that root, saves the previous active window root and focused leaf, and scopes focus, mouse, and command routing to the root subtree (plus any views attached with `HostAttachViewToActiveModal`).

`Application.ShowDialog(App, ModalId, X, Y, Width, Height)` registers a fresh root host view, shows it as the active modal, saves the same return context, and returns its `ViewId`. Closing that modal automatically unregisters the owned root subtree.

`Application.CloseModal(App)` pops the active modal frame (no-op on an empty stack). On close the host restores the saved focused leaf when it still exists, otherwise re-activates the saved window root, then requests redraw for affected views. Nested modals restore one frame at a time.

`Application.HostEnterModal(App, ModalId)` / `Application.HostLeaveModal(App)` remain low-level stack primitives without automatic view ownership. `Application.QueryModalDepth(App)` returns the stack depth.

`Application.HostSetActiveModalResult(App, ResultCode)` stores the active frame's result after validating `ResultCode`: `1` Accept, `2` Cancel, values `>= 1000` application-defined. Invalid codes or an empty stack are runtime errors. Call before `CloseModal` when the outcome matters.

`Application.HostBindCommandToActiveModal` binds shortcuts for the top modal frame only. Enter/Escape dismissal is wired through modal-local command bindings and `OnCommand` until dedicated default/cancel host APIs ship.

When a modal scope is active, Tab / Shift+Tab traversal is limited to scoped views, mouse events outside scoped rectangles are suppressed, and view-local or global key / command dispatch is blocked while focus is outside the scope. `HostBindCommandToActiveModal` bindings always dispatch.

### Host view handles

See [Views and focus](views.md) for tree layout, clipping, focus traversal, and paint order.

`Application.HostRegisterView(App, X, Y, Width, Height)` returns an opaque **`ViewId`** owned by the host (see [ViewId type (decided)](#viewid-type-decided) under Native TUI testing API). `Width` and `Height` must both be greater than zero; view and widget creation and `Application.HostSetViewRect` reject invalid dimensions before mutating host state. Pass the handle to `Application.HostUnregisterView`, `Application.HostPushChildView`, `Application.HostSetViewRect`, `Application.HostSetViewParent`, `Application.HostSetViewVisible`, `Application.HostSetViewEnabled`, `Application.HostRegisterOnViewPaint`, and `Application.HostBindCommandToView`.

`Application.HostPushChildView(App, ViewId)` currently marks the view selectable and a Tab stop. Tab / Shift+Tab candidates are derived from retained tree order and skip hidden, disabled, fully clipped, non-selectable, and non-Tab-stop views. `Application.QueryFocusedViewId(App)` returns `Option of ViewId` for the focused leaf (`None` when none). Groups retain their current child along the active focus path internally.

Root views use absolute terminal coordinates. `Application.HostSetViewParent(App, ViewId, Parent)` reparents a view under `Some(Parent)`; pass `None` to detach it back to the root list. Reparenting preserves the current absolute terminal rectangle. After a view has a parent, `Application.HostSetViewRect(App, ViewId, X, Y, Width, Height)` interprets `X` and `Y` relative to that parent. Each node resolves one absolute rectangle and effective ancestor clip used consistently by paint, hit-testing, damage, focus eligibility, and queries. Sibling order defines z-order, and `Application.ShowModal` scopes to a root view subtree.

`Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)` registers a local paint handler for one view. During hosted redraw, the host first runs global `OnPaint` when present and then traverses retained roots depth-first. For each exposed damaged view it paints the native widget underlay, invokes the local handler, paints child subtrees, and then paints overlays. The callback receives local `Bounds` (`0, 0, width, height`); Console coordinates are relative to that view and writes are enforced against its effective clip. `Application.QueryViewRect` continues to return the absolute rectangle before clipping.

### Host widgets

Host widget views are painted entirely in Rust and satisfy the `Application.Run` paint prerequisite on their own. When the visible frame comes from widgets only, `ApplicationHandlers.OnPaint` may be an empty no-op procedure (see `apps/ide/src/shell.fpas`).

`Application.HostCreateSolidFillView(App, X, Y, Width, Height, FillColor, TextColor, FillChar)` registers a host-managed view whose background is painted directly in Rust. `FillColor` is required and uses packed CRT color indices (`0..=15`, same constants as `Std.Console`). `TextColor` and `FillChar` are optional:

- `TextColor := None`, `FillChar := None` — solid fill with spaces using `FillColor` for both foreground and background.
- `FillChar := Some('.')` — tile the rectangle with one character. When `TextColor` is `None`, the host uses `LightGray` (`7`) on top of `FillColor`.
- `TextColor := Some(C)` without a fill character has no effect beyond the space fill.

Widget views participate in the same z-order and damage tracking as Pascal `OnViewPaint` handlers. When both are registered on one view, the host paints the native widget base first and then invokes that view's Pascal handler. Widget overlays such as an open menu popup paint after local handlers so content cannot cover the overlay.

#### `MenuBarItem` and `MenuBarStyle`

`MenuBarItem` is a declarative record:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `Label` | `string` | Visible menu text |
| `Shortcut` | `string` | Alt+letter shortcut (case-insensitive). Use `''` when none. The matching letter in `Label` is drawn in `MenuBarStyle.AccelFg`. |
| `Enabled` | `boolean` | When `false`, drawn disabled and ignores clicks |
| `CommandId` | `integer` | Dispatched through `OnCommand` on click; use `-1` for non-clickable labels |
| `Submenu` | `array of MenuPopupItem` | Pull-down entries. Use `[]` for top-level commands without a submenu. |

`MenuPopupItem` is a declarative pull-down record:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `Label` | `string` | Visible menu text |
| `Shortcut` | `string` | Letter shortcut while the popup is open. Use `''` when none. |
| `Enabled` | `boolean` | When `false`, drawn disabled and ignores activation |
| `CommandId` | `integer` | Dispatched through `OnCommand` on activation |
| `Separator` | `boolean` | When `true`, draws a horizontal rule and ignores activation. Defaults to `false`. |

`MenuBarStyle` supplies CRT color indices (`0..=15`) for `BarBg`, `BarFg`, `AccelFg`, `HighlightBg`, `HighlightFg`, and `DisabledFg`.

`Application.HostCreateMenuBarView(App, X, Y, Width, Height, Items, Style)` registers a host-managed menu bar. Rust paints the bar, open pull-downs, and performs hit-testing. `Application.HostSetMenuBarItems(App, ViewId, Items)` replaces the model at runtime.

##### Menu bar input priority

During hosted keyboard dispatch (`TuiHostProcessNext` / `Application.Run`), the host evaluates keys in this order:

1. **Tab / Shift+Tab** traversal through eligible retained views.
2. **Menu bar widget** routing on the topmost eligible menu bar in paint order (see **Navigation** below). During a modal scope, only menu bars inside that scope are eligible.
3. **Command bindings** resolved from view-local maps, modal maps, then `Application.HostBindCommand(App, Key, CommandId)`.
4. **`OnKeyPressed`** when registered.

During hosted mouse dispatch, the host evaluates events in this order:

1. Resolve a typed retained route from pointer capture or the topmost enabled clipped view.
2. **Modal scope** suppression when no routed target belongs to the active modal.
3. **Menu bar widget** routing, including open popup rectangles. Menu bars take priority over other host widgets underneath the pointer, but only widgets inside the active modal scope are eligible.
4. Move focus on pointer-down when the target is selectable.
5. Retained **control** widgets (buttons, inputs, list boxes, …) on pointer-down or wheel.
6. **`OnMouse`** when registered.

When the terminal gains focus while a menu-bar item is already hovered, the host opens that item's pull-down on the same activation path used for a bar click (integrated terminals often deliver focus before the first mouse-down). Terminal focus loss clears menu-bar hover and open pull-down state.

Terminal focus loss and view removal release pointer capture. Capture continues routing pointer
events to its owner outside the original hit rectangle until released.

Menu-bar widget routing happens **before** global command bindings and `OnKeyPressed` / `OnMouse`. Keys consumed for hover or submenu redraw return process tag **`21`**; mouse hover redraws may return tag **`5`**. Activated menu commands still dispatch through **`OnCommand`** (tag **`16`** when a handler is registered).

##### Menu bar navigation

**Top-level activation**

- **Alt+`Shortcut`** (bar item `Shortcut` field): enters menu mode, highlights the matching enabled top-level item, opens its `Submenu` when non-empty, otherwise dispatches `OnCommand` when `CommandId >= 0`.
- **F10** (unmodified): enters menu mode on the first enabled top-level item and opens its submenu when present.
- **Mouse hover** updates the highlighted enabled item; **mouse down** on a top-level item toggles/opens its submenu or dispatches `OnCommand` when `CommandId >= 0`.

**Menu mode** (after F10, Alt+shortcut, or bar interaction)

- **Left** / **Right**: cycle across enabled top-level items (wrap). When the highlighted item has a `Submenu`, the host opens or switches to that pull-down.
- **Down**: opens the submenu of the highlighted top-level item when it has entries.
- **Escape**: exits menu mode and clears the bar highlight.

**Open submenu**

- **Up** / **Down**: move the highlighted pull-down entry, skipping disabled rows and separators.
- **Enter**: activates the highlighted entry when it is enabled and not a separator (`OnCommand` when `CommandId >= 0`).
- **Escape**: closes the submenu and returns to menu mode on the same bar item.
- **Letter key** (popup `Shortcut`, no modifiers) or **Alt+letter**: activates the first matching enabled popup entry.

**Mouse on open submenu**

- **Mouse down** inside the popup activates the clicked enabled entry.
- **Mouse down** outside the popup closes it; leaving the bar clears hover when appropriate.

#### `StatusBarSegment` and `StatusBarStyle`

`StatusBarSegment` is a declarative record:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `Text` | `string` | Visible status text |
| `AlignRight` | `boolean` | When `true`, anchor the segment to the right edge of the bar |

`StatusBarStyle` supplies CRT color indices (`0..=15`) for `BarBg` and `BarFg`.

`Application.HostCreateStatusBarView(App, X, Y, Width, Height, Segments, Style)` registers a host-managed status bar. Rust paints left-aligned segments in order, then right-aligned segments from the right edge inward. `Application.HostSetStatusBarSegments(App, ViewId, Segments)` replaces the model at runtime.

Command resolution is ordered from most local to least local: when a focused host-managed view exists, the host first checks command maps bound to that view and then its ancestors, then the active modal frame's command map, and finally the global command map installed through `Application.HostBindCommand(App, Key, CommandId)`.

---

## See also

- [Modals and dialogs](modals.md)
- [Views and focus](views.md)
- [Hosted dispatch overview](README.md)
- [Handlers](handlers.md)
- [Native testing](testing.md#viewid-type-decided)
