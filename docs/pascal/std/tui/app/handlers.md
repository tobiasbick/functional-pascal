# Handlers

## Current registration model

Pascal can register hosted handlers in four equivalent ways before `**Application.Run(App)`**:

1. **Bundle form** with `**Application.Configure(App, Handlers)`** using the shipped record type `**ApplicationHandlers`**.
2. **Explicit form** with the `**Application.HostRegisterOn*`** routines.
3. **Per-view paint form** with `**Application.HostRegisterOnViewPaint(App, ViewId, OnViewPaint)`**.
4. **Host widget form** with `**Application.HostCreateSolidFillView**`, `**Application.HostCreateMenuBarView**`, or `**Application.HostCreateStatusBarView**` (create widgets before `**Run`**).

The most recent configuration wins per slot. `**Application.Configure`** replaces the current hosted handler set with the record fields from `**ApplicationHandlers`**. View-local paint handlers are tracked separately per host view. Host widget views are tracked separately in the view registry.

**Required** for a minimal hosted run: at least one global `**OnPaint`**, at least one local view paint handler, or at least one host widget view. In bundle form, `**ApplicationHandlers.OnPaint`** is optional; leave it unset for widget-only applications.

### `ApplicationHandlers`

Shipped record fields:


| Slot           | Required | Role                                                                                                                            |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `OnPaint`      | no       | `Option of procedure(App: Application)` — full logical **frame** paint. Use `None` or omit it when retained views/widgets paint the frame. |
| `OnKeyPressed` | no       | `Option of function(App: Application; Key: Std.Console.KeyEvent): boolean` — key / text input.                                 |
| `OnResize`     | no       | `Option of procedure(App: Application; NewSize: Size)` — terminal size changed (coalesced by the host).                        |
| `OnIdleMilliseconds` | no | Idle interval in milliseconds. `<= 0` disables idle callbacks.                                                                  |
| `OnIdle`       | no       | `Option of procedure(App: Application)` — host-invoked when no input arrived for the configured idle interval.                 |
| `OnExit`       | no       | `Option of procedure(App: Application; Reason: ExitReason)` — last user hook before terminal restore.                          |
| `OnMouse`      | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — mouse input (click, scroll, move).                        |
| `OnPaste`      | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — bracketed-paste content (`Event.text`). Best-effort; requires `Std.Console.EnablePaste` on the active session. |
| `OnFocusGained` | no      | `Option of procedure(App: Application; Event: Std.Console.Event)` — terminal gained focus. Best-effort / optional on many terminals. |
| `OnFocusLost`  | no       | `Option of procedure(App: Application; Event: Std.Console.Event)` — terminal lost focus. Best-effort / optional on many terminals. |
| `OnActivate`   | no       | `Option of procedure(App: Application)` — an eligible retained view gained focus through traversal or pointer-down. Fires after `OnDeactivate` when there was a prior focused leaf. |
| `OnDeactivate` | no       | `Option of procedure(App: Application)` — the previous focused retained view lost focus. Fires before `OnActivate` for the new leaf. |
| `OnCommand`    | no       | `Option of procedure(App: Application; CommandId: integer)` — a host-resolved keyboard shortcut fired. Command ids are application-defined integers bound through global, view-local, or modal-local command maps. |

Example:

```pascal
var Handlers: ApplicationHandlers := record
  OnPaint := Some(OnPaint);
  OnKeyPressed := Some(OnKeyPressed);
  OnIdleMilliseconds := 16;
  OnIdle := Some(OnIdle);
  OnExit := Some(OnExit);
  OnMouse := Some(OnMouse);
end;
```


---

## `OnExit`

- **When:** Invoked **once** after the host decides to stop the loop and **before** terminal restore (`**Close`** semantics).
- **Purpose:** teardown of user state (runs once per shutdown).
- **Ordering:** `**OnExit`** runs **after** the last `**OnPaint`** / input handler for that run; no further `**On*`** run after `**OnExit`** except what the implementation documents for catastrophic failure paths.

---

## Redraw and paint

- **Model:** **invalidation**, not “call `**OnPaint`** every host tick”. The host sets an internal **redraw pending** flag when `**Application.RequestRedraw`** is called, when `**OnResize`** fires, when `**Application.Run`** starts (the host auto-requests the first redraw), and when the backend signals damage the host maps to a redraw. Multiple requests **coalesce** to **one** hosted flush.
- `**OnPaint`:** Performs a **full logical frame** draw (entire buffer for the app). The Rust host now batches each hosted paint into one deferred back-buffered present and may restrict terminal diff/flush work to tracked dirty regions plus the console mutations recorded during that frame.
- `**OnViewPaint`:** Performs view-local paint for one host-managed view. The host traverses each view depth-first as native underlay, local handler, child subtrees, then overlay. `**Bounds`** is local (`x = 0`, `y = 0`), and CRT operations such as `**GotoXY(1, 1)`** address the view's top-left cell. Writes are hard-clipped to the view's effective ancestor clip, and Console window/cursor state is restored after the callback.
- **Relation to `RedrawPending`:** In dispatch mode, user code **typically does not poll** `**RedrawPending`**; the host invokes `**OnPaint`** when a frame is due. If both APIs coexist during transition, the spec for `**RedrawPending**` in hosted mode is: host consumes the pending flag when entering `**OnPaint**` (aligned with today’s “consume once” semantics).

---

## `OnIdle`

- Optional. If the idle interval is **greater than zero**, the host may call `**OnIdle(App)`** when no input was available for that interval and no higher-priority work ran. Used for caret blink, status clocks, etc.
- `**OnIdle`** must not block on input (same **reentrancy** rules as `[docs/future/tui-application-framework.md](../../../../future/tui-application-framework.md)` Phase 0).

---

## `OnKeyPressed`

- **When:** Fired once for each key or text-input event delivered by the host, in the order the host dequeues events from the terminal. Within a single dispatch turn, at most one key event is dispatched before control returns to the host loop (no batching of multiple keys in one handler call).
- **Ordering relative to other handlers:** `OnResize` events that arrive before the key in the host queue are coalesced and dispatched first; `OnPaint` runs after input dispatch if a redraw is pending.
- **Return value:** `true` means the key was **consumed**; `false` means it was not consumed. The low-level `HostProcessNext` reports these outcomes as tags `1` and `22` respectively. `OnKeyPressed` is the final application fallback in the current routing order, so no later current handler runs after either result.
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

There must be **at most one** active `**Application.Run`** (or equivalent hosted loop) per process for a given session handle. Modal nesting is expressed through `**Application.ShowModal**` / `**Application.ShowDialog**` / `**Application.CloseModal**`; nested `**Run**` remains **forbidden**. See [Modals and dialogs](modals.md).

## See also

- [Modals and dialogs](modals.md)
- [Views and focus](views.md)

- [Types](types.md)
- [Lifecycle](lifecycle.md)
- [Hosted dispatch overview](README.md)
