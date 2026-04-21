# `Std.Tui` — dispatch-mode application (target)

**Status:** target specification for the Rust-hosted event loop and `On*` handlers described in `[docs/future/tui-application-framework.md](../../future/tui-application-framework.md)`. **`Application.Host*`** dispatch helpers are **registered and lowered**, **`ApplicationHandlers`** / **`Application.Configure(App, Handlers)`** are available as the bundled registration surface, and **`Application.Run(App)`** is available as the hosted loop entrypoint. `OnIdle` remains available through both `Application.HostRegisterOnIdle(App, Milliseconds, OnIdle)` and the bundle field pair `OnIdleMilliseconds` + `OnIdle`. The poll-style API in `[tui.md](tui.md)` remains available for programs that do not use hosted dispatch.

**Maintenance (implementers only):** when this mode ships, register types and routines in `[loaded/tui.rs](../../../crates/fpas-sema/src/std_registry/loaded/tui.rs)` and keep this file aligned with that registry (see root `[AGENTS.md](../../../AGENTS.md)`).

---

## VM bridge (Phase 3–4)

These `[fpas_bytecode::Intrinsic](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)` variants drive `fpas_std::TuiHost` from the VM. In Pascal they appear as **`Std.Tui.Application.Host*`** (see table below); stack order matches other TUI intrinsics: pass `Application`, duplicate with the bytecode `Dup` opcode when the handle is needed again.


| Intrinsic                     | Stack (bottom → top)                             | Result                                                                                                                                                                                                                                                                              |
| ----------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TuiHostPollNext`             | `Application`                                    | `Option<Std.Tui.TuiEvent>` with host resize coalescing.                                                                                                                                                                                                                             |
| `TuiHostRegisterOnKeyPressed` | `Application`, `function`                        | Registers `function (Application, Std.Console.KeyEvent): boolean` for invoke.                                                                                                                                                                                                       |
| `TuiHostInvokeOnKeyPressed`   | `Application`, `Std.Console.KeyEvent`            | Calls the registered function; pushes `boolean` (`consumed`).                                                                                                                                                                                                                       |
| `TuiHostRegisterOnResize`     | `Application`, `function`                        | Registers `procedure (Application, Std.Tui.Size)` (arity 2).                                                                                                                                                                                                                        |
| `TuiHostProcessNext`          | `Application`, `max_spins` (`integer`, top)      | Spins up to `max_spins` (clamped to `4096`, minimum one iteration) through `poll_event` + host ingest, then dispatches **at most one** `HostEvent`. Pushes `integer`: `0` no event, `1` key dispatched, `2` resize dispatched, `3` key without handler, `4` resize without handler, `5` mouse dispatched, `7` mouse without handler. |
| `TuiHostRegisterOnPaint`      | `Application`, `function`                        | Registers `procedure (Application)` (arity 1).                                                                                                                                                                                                                                      |
| `TuiHostRegisterOnIdle`       | `Application`, `integer`, `function`             | Registers `procedure (Application)` plus an idle interval in milliseconds. `Milliseconds <= 0` disables idle callbacks.                                                                                                                                                             |
| `TuiHostDispatchRedraw`       | `Application`                                    | If redraw is pending: runs registered `OnPaint` after `take_redraw_pending`, or clears the flag with tag `6` when no handler. Pushes `integer`: `0` not pending, `5` paint ran, `6` cleared without handler.                                                                        |
| `TuiHostRunLoop`              | `Application`, `max_iterations` (`integer`, top) | Bounded host loop: each iteration runs the same work as `TuiHostDispatchRedraw` then `TuiHostProcessNext` with a fixed inner `max_spins` of `64`. After each iteration, if `TuiHostRequestQuit` was observed, the loop stops and the quit flag is cleared. Otherwise stops when both steps would be idle (`0`). `max_iterations` is clamped to `0..=1_000_000`. Pushes `()`. |
| `TuiHostRequestQuit`          | `Application`                                    | Sets a flag read by `TuiHostRunLoop` after each iteration. Does not push a value.                                                                                                                                                                                                 |
| `TuiHostRegisterOnExit`       | `Application`, `function`                        | Registers `procedure (Application, ExitReason)` for a future hosted `Run` / `OnExit` path. Current bounded `HostRunLoop` does **not** invoke it yet.                                                                                                                               |
| `TuiHostRegisterOnMouse`      | `Application`, `function`                        | Registers `procedure (Application, Std.Console.Event)` (arity 2) for host mouse-event dispatch.                                                                                                                                                                                     |
| `TuiApplicationConfigure`     | `Application`, `ApplicationHandlers`             | Applies a bundled hosted-dispatch configuration. Replaces the current hosted handlers with the record fields from `ApplicationHandlers`; `OnPaint` is required, optional handlers use `Some(Handler)` or `None`, and `OnIdleMilliseconds <= 0` disables idle callbacks.        |
| `TuiApplicationRun`           | `Application`                                    | Hosted loop entrypoint. Requires a previously registered `OnPaint` handler, auto-requests the first redraw, blocks until `Application.HostRequestQuit(App)` is observed **or** the host stops the active run, records `ExitReason.UserQuit`, `ExitReason.HostStop`, `ExitReason.HostAndUserStop`, or `ExitReason.HostShutdown`, invokes `OnExit` when registered, and performs `Application.Close` semantics before returning. Pushes `()`. |

### Pascal names (registry + compiler)

| Pascal `Std.Tui` call | Maps to intrinsic |
| ----------------------- | ----------------- |
| `Application.HostPollNext(App)` | `TuiHostPollNext` |
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
| `Application.Configure(App, Handlers)` | `TuiApplicationConfigure` |
| `Application.Run(App)` | `TuiApplicationRun` |

Samples: [`examples/pascal/tui/host_dispatch_minimal.fpas`](../../../examples/pascal/tui/host_dispatch_minimal.fpas) (one `HostProcessNext` step), [`examples/pascal/tui/host_dispatch_paint.fpas`](../../../examples/pascal/tui/host_dispatch_paint.fpas) (register `OnPaint` + `HostDispatchRedraw`), [`examples/pascal/tui/host_dispatch_quit.fpas`](../../../examples/pascal/tui/host_dispatch_quit.fpas) (`HostRequestQuit` from `OnPaint` + `HostRunLoop`).

**Bytecode discriminants** (authoritative enum: [`Intrinsic`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)): **255** `TuiHostPollNext`, **256** `TuiHostRegisterOnKeyPressed`, **257** `TuiHostInvokeOnKeyPressed`, **258** `TuiHostRegisterOnResize`, **259** `TuiHostProcessNext`, **260** `TuiHostRegisterOnPaint`, **261** `TuiHostDispatchRedraw`, **262** `TuiHostRunLoop`, **263** `TuiHostRequestQuit`, **264** `TuiHostRegisterOnExit`, **265** `TuiApplicationRun`, **266** `TuiHostRegisterOnIdle`, **267** `TuiApplicationConfigure`, **268** `TuiHostRegisterOnMouse`.

`Application.Close` clears registered host handlers (`OnKeyPressed`, `OnResize`, `OnPaint`, `OnIdle`, `OnExit`, `OnMouse`), resets the host pump state, and closes the session as today.

---

## Relationship to the poll-style API

Today, `[tui.md](tui.md)` documents `**Application.ReadEvent`**, `**ReadEventTimeout`**, `**PollEvent**`, and redraw helpers. The dispatch model replaces that pattern for full applications: the runtime owns the blocking loop and calls user `**On***` procedures. Poll-style entry points are **removed or narrowed** once dispatch mode exists (project policy: no backward compatibility requirement).

Dispatch-mode names use the `**On` prefix** so they do not collide with legacy names such as console `**KeyPressed`** (boolean poll).

---

## Session and entry


| Step                | Meaning                                                                                                                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Application.Open`  | Same session semantics as today: acquire terminal state (raw mode, alternate screen when applicable).                                                                 |
| `Application.Run`   | Start the **hosted** main loop for the given `Application` handle. Register handlers first with `Application.Configure(App, Handlers)` or the explicit `Application.HostRegisterOn*` helpers; `OnPaint` is required. The loop auto-requests the first redraw and blocks until the application requests quit. |
| `Application.Close` | Release the session. After `**Application.Run`** completes successfully, the host **must** have restored the session as if `**Close`** ran (see **Lifecycle** below). |

**Current Pascal surface:** `**Application.Configure(App, Handlers)`** lowers to a dedicated intrinsic and writes the bundled hosted handlers (`**OnPaint`** required; optional handlers use `**Some(...)`** / `**None`**). `**Application.Run(App)`** then uses whichever handlers were registered last, whether through `**Configure`** or the explicit `**Application.HostRegisterOn*`** helpers. `**TuiHostRunLoop**` (**262**) remains available as the low-level bounded stepping helper for tests and explicit host experimentation.

### Lifecycle (normative)

1. User calls `**Application.Open`** → receives `**App`**.
2. User registers handlers with `**Application.Configure(App, Handlers)`** or `**Application.HostRegisterOn*`** (`**OnPaint`** required, others optional).
3. User calls `**Application.Run(App)`**.
4. While running, the host dispatches `**On*`** handlers on the **main VM thread** only (see `[parallel-vm.md](../../rust/parallel-vm.md)`).
5. When the application requests quit, the host records `**ExitReason.UserQuit`**. If the active hosted session is stopped by low-level host control during `**Run`** (for example `**Application.Close(App)`** is invoked while the run is still active), the host records `**ExitReason.HostStop`**. If both are requested in the same turn, the host records `**ExitReason.HostAndUserStop`**. If the VM enters global shutdown while the hosted run is active (for example after a concurrent task failure), the host records `**ExitReason.HostShutdown`**. In every case it invokes `**OnExit(App, Reason)`** once if that handler is provided, then **performs `Application.Close(App)`** (or equivalent) so the program must **not** call `**Close`** again for the same successful `**Run`**.

If `**Run`** is never called, the program keeps today’s obligation: `**Open**` / `**Close**` pairing without `**Run**`.

---

## Current registration model

Pascal can register hosted handlers in two equivalent ways before `**Application.Run(App)`**:

1. **Bundle form** with `**Application.Configure(App, Handlers)`** using the shipped record type `**ApplicationHandlers`**.
2. **Explicit form** with the `**Application.HostRegisterOn*`** routines.

The most recent configuration wins per slot. `**Application.Configure`** replaces the current hosted handler set with the record fields from `**ApplicationHandlers`**.

**Required** for a minimal app: at least `**OnPaint`** (the runtime rejects `**Run`** otherwise). Other slots are optional.

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

### `ExitReason` (target)

Enum describing why the hosted loop stopped (`**Std.Tui.ExitReason`**). **Registry:** the type and variants `**UserQuit**`, `**HostStop`**, `**HostAndUserStop**`, `**HostShutdown**` are registered in [`loaded/tui.rs`](../../../crates/fpas-sema/src/std_registry/loaded/tui.rs) and known to the compiler enum tables. **VM:** [`Application.Run`](../../../crates/fpas-vm/src/vm/execute/io/tui_run.rs) records `**last_exit_reason**`, invokes the registered `**OnExit**`, and then performs close semantics. The current hosted loop reports `**UserQuit`** when `**Application.HostRequestQuit(App)`** ends the run, `**HostStop`** when low-level code stops the active hosted session during `**Run`**, `**HostAndUserStop`** when both stop signals are present in the same turn, and `**HostShutdown`** when VM global shutdown is requested while the hosted run is active.


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

- **Model:** **invalidation**, not “call `**OnPaint`** every host tick”. The host sets an internal **redraw pending** flag when `**Application.RequestRedraw`** is called, when `**OnResize`** fires, when `**OnStartup`** completes (implementation may auto-request redraw once), and when the backend signals damage the host maps to a redraw. Multiple requests **coalesce** to **one** `**OnPaint`** per logical flush.
- `**OnPaint`:** Performs a **full frame** draw (entire buffer for the app). **Damage rectangles** and partial updates are **Rust-internal** optimizations later; the FP contract stays **full paint** until Phase 7 narrows it.
- **Relation to `RedrawPending`:** In dispatch mode, user code **typically does not poll** `**RedrawPending`**; the host invokes `**OnPaint`** when a frame is due. If both APIs coexist during transition, the spec for `**RedrawPending**` in hosted mode is: host consumes the pending flag when entering `**OnPaint**` (aligned with today’s “consume once” semantics).

---

## `OnIdle`

- Optional. If the idle interval is **greater than zero**, the host may call `**OnIdle(App)`** when no input was available for that interval and no higher-priority work ran. Used for caret blink, status clocks, etc.
- `**OnIdle`** must not block on input (same **reentrancy** rules as `[docs/future/tui-application-framework.md](../../future/tui-application-framework.md)` Phase 0).

---

## Threading and reentrancy

Same as Phase 0 of the framework plan: `**On*`** only on the **main VM thread**; no `**ReadEvent`** / `**Run`** from inside handlers; `**RequestRedraw`** and non-blocking queries are allowed. Spawned tasks must not touch TUI/console state without a future synchronized API.

---

## Optional and best-effort events

Handlers that depend on terminal or OS capability (**key release**, **paste**, **focus**) are specified per `**HostEvent`** mapping in implementation docs; if a backend cannot emit an event, the corresponding `**On`*** (when added in later phases) is **not called** or is documented as a no-op. Phase 6 of the framework plan tracks `**OnKeyReleased`** / `**OnKeyDown`** honesty.

---

## Single entry point rule

There must be **at most one** active `**Application.Run`** (or equivalent hosted loop) per process for a given session handle; nested `**Run`** is **forbidden** until a later spec defines modal nesting.