# `Std.Tui` — dispatch-mode application

**Status:** current specification for the Rust-hosted event loop and `On*` handlers described in `[docs/future/tui-application-framework.md](../../future/tui-application-framework.md)`. **`Application.Host*`** dispatch helpers are **registered and lowered**, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** are available as the bundled registration surface, **`Application.Run(App)`** is available as the hosted loop entrypoint, and the current Phase 7 structure layer includes **`Std.Tui.Rect`**, **`Application.HostSetViewParent`**, **`Application.HostRegisterOnViewPaint`**, **`Application.HostBindCommandToView`**, **`Application.HostBindCommandToActiveModal`**, **`Application.ShowModal`**, **`Application.ShowDialog`**, and **`Application.CloseModal`**. `OnIdle` remains available through both `Application.HostRegisterOnIdle(App, Milliseconds, OnIdle)` and the bundle field pair `OnIdleMilliseconds` + `OnIdle`. Poll-style `Application.ReadEvent` / `Application.PollEvent` are **not** part of the current surface — use hosted dispatch instead (see `[tui.md](tui.md)`).

**Maintenance (implementers only):** keep the types and routines in [`loaded/tui/`](../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) aligned with this file (see root [AGENTS.md](../../../AGENTS.md)).

---

## VM bridge (Phase 3–4)

These `[fpas_bytecode::Intrinsic](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)` variants drive `fpas_std::TuiHost` from the VM. In Pascal they appear as **`Std.Tui.Application.Host*`** (see table below); stack order matches other TUI intrinsics: pass `Application`, duplicate with the bytecode `Dup` opcode when the handle is needed again.


| Intrinsic                     | Stack (bottom → top)                             | Result                                                                                                                                                                                                                                                                              |
| ----------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TuiHostRegisterOnKeyPressed` | `Application`, `function`                        | Registers `function (Application, Std.Console.KeyEvent): boolean` for invoke.                                                                                                                                                                                                       |
| `TuiHostInvokeOnKeyPressed`   | `Application`, `Std.Console.KeyEvent`            | Calls the registered function; pushes `boolean` (`consumed`).                                                                                                                                                                                                                       |
| `TuiHostRegisterOnResize`     | `Application`, `function`                        | Registers `procedure (Application, Std.Tui.Size)` (arity 2).                                                                                                                                                                                                                        |
| `TuiHostProcessNext`          | `Application`, `max_spins` (`integer`, top)      | Spins up to `max_spins` (clamped to `4096`, minimum one iteration) through the hosted input pump, then dispatches **at most one** supported hosted event. Pushes `integer`: `0` no event, `1` key dispatched, `2` resize dispatched, `3` key without handler, `4` resize without handler, `5` mouse dispatched, `7` mouse without handler, `8` paste dispatched, `9` paste without handler, `10` focus-gained dispatched, `11` focus-gained without handler, `12` focus-lost dispatched, `13` focus-lost without handler, `14` Tab traversal advanced (focus moved forward), `15` Shift+Tab traversal retreated (focus moved backward), `16` command dispatched, `17` command bound without handler, `18` key blocked by the active modal scope, `19` mouse blocked by the active modal scope, `20` command blocked by the active modal scope. Tags `14`/`15` are only emitted when the focus chain has eligible views; with an active modal scope, traversal is limited to the attached modal views. |
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
| `TuiHostRegisterOnActivate`   | `Application`, `function`                        | Registers `procedure (Application)` (arity 1) for host-managed view focus-gained dispatch. Fires when Tab / Shift+Tab advances focus to a new view in the focus chain.                                                                                                             |
| `TuiHostRegisterOnDeactivate` | `Application`, `function`                        | Registers `procedure (Application)` (arity 1) for host-managed view focus-lost dispatch. Fires when a view in the focus chain loses focus due to Tab / Shift+Tab traversal.                                                                                                        |
| `TuiHostRegisterOnCommand`    | `Application`, `function`                        | Registers `procedure (Application, integer)` (arity 2) for host-resolved command dispatch. The integer argument is the command id bound to the shortcut.                                                                                                                           |
| `TuiHostBindCommand`          | `Application`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event (kind, character, and modifier flags) to a command id. Rebinding the same key replaces the previous command.                                                                                                                                            |
| `TuiHostBindCommandToView`    | `Application`, `integer`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event to a command id for one host-managed view. The binding is eligible when that view or one of its descendants currently has focus.                                                                                                                        |
| `TuiHostBindCommandToActiveModal` | `Application`, `Std.Console.KeyEvent`, `integer` | Binds a complete key event to a command id for the active modal frame only. The binding disappears when that modal frame is closed.                                                                                                                                             |
| `TuiHostEnterModal`           | `Application`, `integer`                         | Pushes an application-defined modal id onto the host modal stack. Does not push a value.                                                                                                                                                                                           |
| `TuiHostLeaveModal`           | `Application`                                    | Pops the active host modal frame, if any. Leaving an empty modal stack is a no-op. Does not push a value.                                                                                                                                                                          |
| `TuiHostModalDepth`           | `Application`                                    | Pushes `integer`: the active modal stack depth.                                                                                                                                                                                                                                    |
| `TuiHostRegisterView`         | `Application`, `integer`, `integer`, `integer`, `integer` | Registers a host-managed view from `x`, `y`, `width`, `height` and pushes an opaque integer handle. Registration order remains the host paint order.                                                                                                                        |
| `TuiHostUnregisterView`       | `Application`, `integer`                         | Removes a host-managed view by handle. Unknown handles are ignored. Does not push a value.                                                                                                                                                                                        |
| `TuiHostPushChildView`        | `Application`, `integer`                         | Appends a host-managed view handle to the focus chain used by Tab / Shift+Tab traversal. Does not push a value.                                                                                                                                                                  |
| `TuiHostQueryFocusedViewId`   | `Application`                                    | Pushes `integer`: the currently focused view handle, or `-1` when no host-managed view is focused.                                                                                                                                                                               |
| `TuiHostAttachViewToActiveModal` | `Application`, `integer`                      | Attaches a host-managed view handle to the currently active modal frame. Attached views define the modal focus/mouse scope for the topmost modal. Does not push a value.                                                                                                      |
| `TuiHostSetViewRect`          | `Application`, `integer`, `integer`, `integer`, `integer`, `integer` | Updates a host-managed view handle to `x`, `y`, `width`, `height`. Unknown handles are ignored. Does not push a value.                                                                                                                                      |
| `TuiHostSetViewParent`        | `Application`, `integer`, `integer`           | Reparents a host-managed view under `parent_view_id`. Pass `-1` to detach the view back to the root list. The view keeps its current absolute terminal rectangle during the reparenting step. Unknown handles are ignored. Does not push a value.                          |
| `TuiHostRegisterOnViewPaint`  | `Application`, `integer`, `function`          | Registers `procedure (Application, integer, Std.Tui.Rect)` (arity 3) as a view-local paint handler for one host-managed view. During hosted redraw, the host invokes it in tree paint order when that view intersects the current damage region.                              |
| `TuiApplicationConfigure`     | `Application`, `ApplicationHandlers`             | Applies a bundled hosted-dispatch configuration. Replaces the current hosted handlers with the record fields from `ApplicationHandlers`; `OnPaint` is required, optional handlers use `Some(Handler)` or `None`, and `OnIdleMilliseconds <= 0` disables idle callbacks.        |
| `TuiApplicationRun`           | `Application`                                    | Hosted loop entrypoint. Requires a previously registered global `OnPaint` handler **or** at least one local view paint handler, auto-requests the first redraw, blocks until `Application.HostRequestQuit(App)` is observed **or** the host stops the active run, records `ExitReason.UserQuit`, `ExitReason.HostStop`, `ExitReason.HostAndUserStop`, or `ExitReason.HostShutdown`, invokes `OnExit` when registered, and performs `Application.Close` semantics before returning. Pushes `()`. |
| `TuiApplicationShowModal`     | `Application`, `integer`, `integer`             | Pushes a modal frame anchored to the given root view. The root view is raised, the modal scope becomes that view subtree (plus any explicitly attached extra views), and focus is moved into that scope when possible. Does not push a value.                               |
| `TuiApplicationShowDialog`    | `Application`, `integer`, `integer`, `integer`, `integer`, `integer` | Registers a new root host view for `x`, `y`, `width`, `height`, shows it as the active modal dialog, and pushes the new root `ViewId` as `integer`. Closing that modal automatically unregisters the owned root subtree.                                                     |
| `TuiApplicationCloseModal`    | `Application`                                    | Pops the active modal frame created by `Application.ShowModal`, `Application.ShowDialog`, or `Application.HostEnterModal`. Leaving an empty modal stack is a no-op. Does not push a value.                                                                                     |

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
| `Application.HostModalDepth(App)` | `TuiHostModalDepth` |
| `Application.HostRegisterView(App, X, Y, Width, Height)` | `TuiHostRegisterView` |
| `Application.HostUnregisterView(App, ViewId)` | `TuiHostUnregisterView` |
| `Application.HostPushChildView(App, ViewId)` | `TuiHostPushChildView` |
| `Application.HostQueryFocusedViewId(App)` | `TuiHostQueryFocusedViewId` |
| `Application.HostAttachViewToActiveModal(App, ViewId)` | `TuiHostAttachViewToActiveModal` |
| `Application.HostSetViewRect(App, ViewId, X, Y, Width, Height)` | `TuiHostSetViewRect` |
| `Application.HostSetViewParent(App, ViewId, ParentViewId)` | `TuiHostSetViewParent` |
| `Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)` | `TuiHostRegisterOnViewPaint` |
| `Application.Configure(App, Handlers)` | `TuiApplicationConfigure` |
| `Application.Run(App)` | `TuiApplicationRun` |
| `Application.ShowModal(App, ModalId, RootViewId)` | `TuiApplicationShowModal` |
| `Application.ShowDialog(App, ModalId, X, Y, Width, Height)` | `TuiApplicationShowDialog` |
| `Application.CloseModal(App)` | `TuiApplicationCloseModal` |

Samples: [`examples/pascal/tui/host_dispatch_minimal.fpas`](../../../examples/pascal/tui/host_dispatch_minimal.fpas) (one `HostProcessNext` step), [`examples/pascal/tui/host_dispatch_paint.fpas`](../../../examples/pascal/tui/host_dispatch_paint.fpas) (register `OnPaint` + `HostDispatchRedraw`), [`examples/pascal/tui/host_dispatch_quit.fpas`](../../../examples/pascal/tui/host_dispatch_quit.fpas) (`HostRequestQuit` from `OnPaint` + `HostRunLoop`).

**Bytecode discriminants** (authoritative enum: [`Intrinsic`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)): **256** `TuiHostRegisterOnKeyPressed`, **257** `TuiHostInvokeOnKeyPressed`, **258** `TuiHostRegisterOnResize`, **259** `TuiHostProcessNext`, **260** `TuiHostRegisterOnPaint`, **261** `TuiHostDispatchRedraw`, **262** `TuiHostRunLoop`, **263** `TuiHostRequestQuit`, **264** `TuiHostRegisterOnExit`, **265** `TuiApplicationRun`, **266** `TuiHostRegisterOnIdle`, **267** `TuiApplicationConfigure`, **268** `TuiHostRegisterOnMouse`, **269** `TuiHostRegisterOnPaste`, **270** `TuiHostRegisterOnFocusGained`, **271** `TuiHostRegisterOnFocusLost`, **272** `TuiHostRegisterOnActivate`, **273** `TuiHostRegisterOnDeactivate`, **274** `TuiHostRegisterOnCommand`, **275** `TuiHostBindCommand`, **276** `TuiHostEnterModal`, **277** `TuiHostLeaveModal`, **278** `TuiHostModalDepth`, **279** `TuiHostRegisterView`, **280** `TuiHostUnregisterView`, **281** `TuiHostPushChildView`, **282** `TuiHostQueryFocusedViewId`, **283** `TuiHostAttachViewToActiveModal`, **284** `TuiHostSetViewRect`, **285** `TuiHostSetViewParent`, **286** `TuiHostRegisterOnViewPaint`, **287** `TuiApplicationShowModal`, **288** `TuiApplicationCloseModal`, **289** `TuiHostBindCommandToView`, **290** `TuiHostBindCommandToActiveModal`, **291** `TuiApplicationShowDialog`.

`Application.Close` clears registered host handlers (`OnKeyPressed`, `OnResize`, `OnPaint`, `OnIdle`, `OnExit`, `OnMouse`, `OnPaste`, `OnFocusGained`, `OnFocusLost`, `OnActivate`, `OnDeactivate`, `OnCommand`), clears local view paint handlers, clears local view command maps, resets the host pump state, clears the view registry (including the focus chain), clears global command bindings, clears the modal stack (including modal-local command bindings), and closes the session as today.

### Modal host state

`Application.ShowModal(App, ModalId, RootViewId)` is the Phase 7 high-level modal surface. It pushes an application-defined modal id together with a root host view, raises that root, and scopes focus, mouse, and command routing to the root subtree. `Application.ShowDialog(App, ModalId, X, Y, Width, Height)` builds on that surface by registering a fresh root host view and returning its `ViewId`; closing that modal automatically unregisters the owned root subtree. `Application.CloseModal(App)` pops the active modal frame and is a no-op when the stack is empty.

`Application.HostEnterModal(App, ModalId)` / `Application.HostLeaveModal(App)` remain the low-level modal-stack primitives, and `Application.HostModalDepth(App)` returns the current stack depth. `Application.HostAttachViewToActiveModal(App, ViewId)` can extend the active modal scope with extra host-managed views beyond the modal root subtree. `Application.HostBindCommandToActiveModal(App, Key, CommandId)` binds shortcuts that only exist while the current modal frame is active. When the active modal has one or more scoped views, Tab / Shift+Tab traversal is limited to those views, mouse events outside their rectangles are suppressed, and key / command dispatch is blocked while focus is on a non-modal view.

### Host view handles

`Application.HostRegisterView(App, X, Y, Width, Height)` returns an opaque integer view handle owned by the host. The current FPAS surface treats that handle as an integer token; pass it back unchanged to `Application.HostUnregisterView(App, ViewId)`, `Application.HostPushChildView(App, ViewId)`, `Application.HostSetViewRect(App, ViewId, X, Y, Width, Height)`, `Application.HostSetViewParent(App, ViewId, ParentViewId)`, `Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)`, and `Application.HostBindCommandToView(App, ViewId, Key, CommandId)`.

`Application.HostPushChildView(App, ViewId)` appends the handle to the focus chain used by Tab / Shift+Tab traversal. `Application.HostQueryFocusedViewId(App)` returns the currently focused handle or `-1` when no host-managed view is focused.

Root views use absolute terminal coordinates. `Application.HostSetViewParent(App, ViewId, ParentViewId)` reparents a view under another view; pass `-1` as `ParentViewId` to detach it back to the root list. Reparenting preserves the current absolute terminal rectangle. After a view has a parent, `Application.HostSetViewRect(App, ViewId, X, Y, Width, Height)` interprets `X` and `Y` relative to that parent. Sibling order defines z-order, and `Application.ShowModal` scopes to a root view subtree.

`Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)` registers a local paint handler for one view. During hosted redraw, the host first runs global `OnPaint` when present and then runs view-local paint handlers in tree paint order for views intersecting the current damage. The `Bounds` argument is the view's absolute terminal rectangle.

Command resolution is ordered from most local to least local: when a focused host-managed view exists, the host first checks command maps bound to that view and then its ancestors, then the active modal frame's command map, and finally the global command map installed through `Application.HostBindCommand(App, Key, CommandId)`.

---

## Relationship to session APIs

`[tui.md](tui.md)` documents the session handle (`Application.Open`, `Application.Size`, `Application.RequestRedraw`) and the hosted entry points (`Application.Configure`, `Application.Run`). Poll-style `Application.ReadEvent`, `Application.ReadEventTimeout`, `Application.PollEvent`, and `Application.RedrawPending` are **not** part of the current surface.

Dispatch-mode names use the `**On` prefix** so they do not collide with legacy names such as console `**KeyPressed`** (boolean poll).

---

## Session and entry


| Step                | Meaning                                                                                                                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Application.Open`  | Same session semantics as today: acquire terminal state (raw mode, alternate screen when applicable).                                                                 |
| `Application.Run`   | Start the **hosted** main loop for the given `Application` handle. Register handlers first with `Application.Configure(App, Handlers)`, the explicit `Application.HostRegisterOn*` helpers, or per-view `Application.HostRegisterOnViewPaint`; at least one global `OnPaint` or local view paint handler is required. The loop auto-requests the first redraw and blocks until the application requests quit. |
| `Application.Close` | Release the session. After `**Application.Run`** completes successfully, the host **must** have restored the session as if `**Close`** ran (see **Lifecycle** below). |

**Current Pascal surface:** `**Application.Configure(App, Handlers)`** lowers to a dedicated intrinsic and writes the bundled hosted handlers (`**OnPaint`** required in bundle form; optional handlers use `**Some(...)`** / `**None`**). `**Application.Run(App)`** then uses whichever handlers were registered last, whether through `**Configure`**, the explicit `**Application.HostRegisterOn*`** helpers, or the per-view `**Application.HostRegisterOnViewPaint`** registrations. `**TuiHostRunLoop**` (**262**) remains available as the low-level bounded stepping helper for tests and explicit host experimentation.

### Lifecycle (normative)

1. User calls `**Application.Open`** → receives `**App`**.
2. User registers handlers with `**Application.Configure(App, Handlers)`**, `**Application.HostRegisterOn*`**, and optionally `**Application.HostRegisterOnViewPaint`** for individual views. A hosted run requires at least one global `**OnPaint`** handler or at least one local view paint handler.
3. User calls `**Application.Run(App)`**.
4. While running, the host dispatches `**On*`** handlers on the **main VM thread** only (see `[parallel-vm.md](../../rust/parallel-vm.md)`).
5. When the application requests quit, the host records `**ExitReason.UserQuit`**. If the active hosted session is stopped by low-level host control during `**Run`** (for example `**Application.Close(App)`** is invoked while the run is still active), the host records `**ExitReason.HostStop`**. If both are requested in the same turn, the host records `**ExitReason.HostAndUserStop`**. If the VM enters global shutdown while the hosted run is active (for example after a concurrent task failure), the host records `**ExitReason.HostShutdown`**. In every case it invokes `**OnExit(App, Reason)`** once if that handler is provided, then **performs `Application.Close(App)`** (or equivalent) so the program must **not** call `**Close`** again for the same successful `**Run`**.

If `**Run`** is never called, the program keeps today’s obligation: `**Open**` / `**Close**` pairing without `**Run**`.

---

## Current registration model

Pascal can register hosted handlers in two equivalent ways before `**Application.Run(App)`**:

1. **Bundle form** with `**Application.Configure(App, Handlers)`** using the shipped record type `**ApplicationHandlers`**.
2. **Explicit form** with the `**Application.HostRegisterOn*`** routines.
3. **Per-view paint form** with `**Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)`**.

The most recent configuration wins per slot. `**Application.Configure`** replaces the current hosted handler set with the record fields from `**ApplicationHandlers`**. View-local paint handlers are tracked separately per host view.

**Required** for a minimal hosted run: at least one global `**OnPaint`** or at least one local view paint handler. In bundle form, `**ApplicationHandlers.OnPaint`** remains required. Other slots are optional.

### `ApplicationHandlers`

Shipped record fields:


| Slot           | Required | Role                                                                                                                            |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `OnPaint`      | **yes**  | Full logical **frame**: draw the entire TUI for this pass.                                                                      |
| `OnKeyPressed` | no       | `Option of function(App: Application; Key: Std.Console.KeyEvent): boolean` — key / text input.                                 |
| `OnResize`     | no       | `Option of procedure(App: Application; NewSize: Size)` — terminal size changed (coalesced by the host).                        |
| `OnIdleMilliseconds` | no | Idle interval in milliseconds. `<= 0` disables idle callbacks.                                                                  |
| `OnIdle`       | no       | `Option of procedure(App: Application)` — host-invoked when no input arrived for the configured idle interval.                 |
| `OnExit`       | no       | `Option of procedure(App: Application; Reason: ExitReason)` — last user hook before terminal restore.                          |
| `OnMouse`      | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — mouse input (click, scroll, move).                        |
| `OnPaste`      | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — bracketed-paste content (`Event.text`). Best-effort; requires `Std.Console.EnablePaste` on the active session. |
| `OnFocusGained` | no      | `Option of procedure(App: Application; Event: Std.Console.Event)` — terminal gained focus. Best-effort / optional on many terminals. |
| `OnFocusLost`  | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — terminal lost focus. Best-effort / optional on many terminals. |
| `OnActivate`   | no       | `Option of procedure(App: Application)` — a view in the host focus chain gained focus (Tab / Shift+Tab traversal). Fires after the previous view's `OnDeactivate` when there was a prior focused view. |
| `OnDeactivate` | no       | `Option of procedure(App: Application)` — a view in the host focus chain lost focus. Fires before `OnActivate` for the new view. |
| `OnCommand`    | no       | `Option of procedure(App: Application; CommandId: integer)` — a host-resolved keyboard shortcut fired. Command ids are application-defined integers bound through global, view-local, or modal-local command maps. |

Example:

```pascal
var Handlers: ApplicationHandlers := record
  OnPaint := OnPaint;
  OnKeyPressed := Some(OnKeyPressed);
  OnIdleMilliseconds := 16;
  OnIdle := Some(OnIdle);
  OnExit := Some(OnExit);
  OnMouse := Some(OnMouse);
end;
```


---

## Types and signatures

Reuse existing types from `**Std.Tui`** and `**Std.Console`** where possible: `**Application**`, `**Size**`, `**Std.Console.KeyEvent**`.

### `Rect`

Record describing the absolute terminal bounds for a host-managed view during local paint dispatch.

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `x` | `integer` | Left edge in terminal cells. |
| `y` | `integer` | Top edge in terminal cells. |
| `width` | `integer` | Width in terminal cells. |
| `height` | `integer` | Height in terminal cells. |

### `ExitReason`

Enum describing why the hosted loop stopped (`**Std.Tui.ExitReason`**). **Registry:** the type and variants `**UserQuit**`, `**HostStop`**, `**HostAndUserStop**`, `**HostShutdown**` are registered in [`loaded/tui/`](../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) and known to the compiler enum tables. **VM:** [`Application.Run`](../../../crates/fpas-vm/src/vm/execute/io/tui_run.rs) records `**last_exit_reason**`, invokes the registered `**OnExit**`, and then performs close semantics. The current hosted loop reports `**UserQuit`** when `**Application.HostRequestQuit(App)`** ends the run, `**HostStop`** when low-level code stops the active hosted session during `**Run`**, `**HostAndUserStop`** when both stop signals are present in the same turn, and `**HostShutdown`** when VM global shutdown is requested while the hosted run is active.


| Variant    | Meaning                                                                                                                                                    |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `UserQuit` | Normal exit requested by the application (for example Escape handled in `**OnKeyPressed**` calling a host **quit** primitive—Phase 3 names the intrinsic). |
| `HostStop` | Host ended the loop for an internal reason (documented per implementation).                                                                                |
| `HostAndUserStop` | Host stop and user quit were both requested in the same dispatch turn; host stop takes precedence but the combined reason is preserved. |
| `HostShutdown` | The VM entered global shutdown while `Application.Run` was active (for example due to a concurrent task failure). |


Future variants (signals, fatal I/O) may extend this enum; handlers must tolerate unknown variants if the language allows exhaustiveness rules.

### Handler signatures (normative)

All procedures run on the **main VM thread**. Parameters use `**App: Application`** for session context.

```pascal
// Conceptual — final Pascal declarations ship with sema registration.

procedure OnStartup(App: Application);

function OnKeyPressed(App: Application; Key: Std.Console.KeyEvent): boolean;
// Returns true if the key was consumed (no further default processing for this event).

procedure OnResize(App: Application; NewSize: Size);

procedure OnViewPaint(App: Application; ViewId: integer; Bounds: Rect);

procedure OnPaint(App: Application);

procedure OnIdle(App: Application);

procedure OnExit(App: Application; Reason: ExitReason);
```

`**OnKeyPressed` return value:** `true` = **consumed**. The host does not promise a second consumer; later phases may use consumption for command routing.

`**OnResize`:** `NewSize` matches `**Application.Size(App)`** after the resize is applied.

---

## `OnExit`

- **When:** Invoked **once** after the host decides to stop the loop and **before** terminal restore (`**Close`** semantics).
- **Veto:** **Not supported** in v1: `**OnExit`** cannot cancel shutdown. It is for teardown of user state only.
- **Ordering:** `**OnExit`** runs **after** the last `**OnPaint`** / input handler for that run; no further `**On*`** run after `**OnExit`** except what the implementation documents for catastrophic failure paths.

---

## Redraw and paint

- **Model:** **invalidation**, not “call `**OnPaint`** every host tick”. The host sets an internal **redraw pending** flag when `**Application.RequestRedraw`** is called, when `**OnResize`** fires, when `**OnStartup`** completes (implementation may auto-request redraw once), and when the backend signals damage the host maps to a redraw. Multiple requests **coalesce** to **one** hosted flush.
- `**OnPaint`:** Performs a **full logical frame** draw (entire buffer for the app). The Rust host now batches each hosted paint into one deferred back-buffered present and may restrict terminal diff/flush work to tracked dirty regions plus the console mutations recorded during that frame.
- `**OnViewPaint`:** Performs view-local paint for one host-managed view. The host runs view-local paint handlers after the global `**OnPaint`** (when present), in tree paint order, and only for views intersecting the current damage. `**Bounds`** is the view's absolute terminal rectangle after parent-relative layout has been resolved.
- **Relation to `RedrawPending`:** In dispatch mode, user code **typically does not poll** `**RedrawPending`**; the host invokes `**OnPaint`** when a frame is due. If both APIs coexist during transition, the spec for `**RedrawPending**` in hosted mode is: host consumes the pending flag when entering `**OnPaint**` (aligned with today’s “consume once” semantics).

---

## `OnIdle`

- Optional. If the idle interval is **greater than zero**, the host may call `**OnIdle(App)`** when no input was available for that interval and no higher-priority work ran. Used for caret blink, status clocks, etc.
- `**OnIdle`** must not block on input (same **reentrancy** rules as `[docs/future/tui-application-framework.md](../../future/tui-application-framework.md)` Phase 0).

---

## `OnKeyPressed`

- **When:** Fired once for each key or text-input event delivered by the host, in the order the host dequeues events from the terminal. Within a single dispatch turn, at most one key event is dispatched before control returns to the host loop (no batching of multiple keys in one handler call).
- **Ordering relative to other handlers:** `OnResize` events that arrive before the key in the host queue are coalesced and dispatched first; `OnPaint` runs after input dispatch if a redraw is pending.
- **Return value:** `true` means the key was **consumed**; the host performs no further default action for this event. `false` passes the event on to any future default routing (Phase 7+).
- **Threading:** Main VM thread only. Must not call blocking event APIs (`Application.ReadEvent`, `Application.Run`) from inside this handler.

---

## `OnResize`

- **When:** Fired when the terminal reports a size change. The host **coalesces** rapid successive resize events — only the final size of a burst is delivered. `NewSize` matches the value `Application.Size(App)` returns immediately after the handler returns.
- **Ordering:** Resize is processed before any key event that arrived in the same batch; the host auto-requests a redraw when a resize fires, so `OnPaint` runs after `OnResize` returns (if no earlier redraw was already pending).
- **Threading:** Main VM thread only. Non-blocking queries (`Application.Size`, `Application.RequestRedraw`) are allowed from inside this handler.

---

## `OnMouse`

- **When:** Fired once per mouse event (button down, button up, cursor move, scroll) as delivered by the terminal backend via `crossterm`. Move events may be frequent; the host does **not** coalesce them.
- **Ordering:** Mouse events are dispatched in arrival order, interleaved with key events. `OnPaint` runs after input dispatch if a redraw was requested during the handler.
- **Event argument:** A `Std.Console.Event` value with the `Mouse` variant; inspect `Event.mouseButton`, `Event.mouseColumn`, `Event.mouseRow`, and related fields.
- **Threading:** Main VM thread only. Same reentrancy rules as `OnKeyPressed`.

---

## `OnPaste`

- **When:** Fired when the terminal delivers a bracketed-paste sequence. **Best-effort** — only fires on terminals that support bracketed-paste mode and only when `Std.Console.EnablePaste` has been called on the active session. On terminals that do not support it this handler is never called.
- **Event argument:** A `Std.Console.Event` value with the `Paste` variant; the pasted text is in `Event.text`.
- **Threading:** Main VM thread only. Same reentrancy rules as `OnKeyPressed`.

---

## `OnFocusGained`

- **When:** Fired when the terminal reports that the application window gained input focus. **Best-effort / optional** — not all terminals emit focus events; if the backend cannot emit this event the handler is never called.
- **Event argument:** A `Std.Console.Event` value with the `FocusGained` variant.
- **Threading:** Main VM thread only. Same reentrancy rules as `OnKeyPressed`.

---

## `OnFocusLost`

- **When:** Fired when the terminal reports that the application window lost input focus. **Best-effort / optional** — same platform caveats as `OnFocusGained`.
- **Event argument:** A `Std.Console.Event` value with the `FocusLost` variant.
- **Threading:** Main VM thread only. Same reentrancy rules as `OnKeyPressed`.

---

## Threading and reentrancy

Same as Phase 0 of the framework plan: `**On*`** only on the **main VM thread**; no `**ReadEvent`** / `**Run`** from inside handlers; `**RequestRedraw`** and non-blocking queries are allowed. Spawned tasks must not touch TUI/console state without a future synchronized API.

---

## Optional and best-effort events

Handlers that depend on terminal or OS capability (**key release**, **paste**, **focus**) are specified per terminal-event mapping in the implementation docs; if a backend cannot emit an event, the corresponding `**On`*** (when added in later phases) is **not called** or is documented as a no-op. Phase 6 of the framework plan tracks `**OnKeyReleased`** / `**OnKeyDown`** honesty.

---

## Single entry point rule

There must be **at most one** active `**Application.Run`** (or equivalent hosted loop) per process for a given session handle. Modal nesting is expressed through `**Application.ShowModal`** / `**Application.CloseModal`**; nested `**Run`** remains **forbidden**.
