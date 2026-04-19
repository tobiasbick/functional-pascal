# `Std.Tui` — dispatch-mode application (target)

**Status:** target specification for the Rust-hosted event loop and `On*` handlers described in `[docs/future/tui-application-framework.md](../../future/tui-application-framework.md)`. **`Application.Host*`** dispatch helpers are **registered and lowered** (Phase 4 — partial); a full **`Application.Run`** bundle, **`OnExit`**, and **`ExitReason`** are still **not** in Pascal. The poll-style API in `[tui.md](tui.md)` remains the default for full programs until `Run` exists.

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
| `TuiHostProcessNext`          | `Application`, `max_spins` (`integer`, top)      | Spins up to `max_spins` (clamped to `4096`, minimum one iteration) through `poll_event` + host ingest, then dispatches **at most one** `HostEvent`. Pushes `integer`: `0` no event, `1` key dispatched, `2` resize dispatched, `3` key without handler, `4` resize without handler. |
| `TuiHostRegisterOnPaint`      | `Application`, `function`                        | Registers `procedure (Application)` (arity 1).                                                                                                                                                                                                                                      |
| `TuiHostDispatchRedraw`       | `Application`                                    | If redraw is pending: runs registered `OnPaint` after `take_redraw_pending`, or clears the flag with tag `6` when no handler. Pushes `integer`: `0` not pending, `5` paint ran, `6` cleared without handler.                                                                        |
| `TuiHostRunLoop`              | `Application`, `max_iterations` (`integer`, top) | Bounded host loop: each iteration runs the same work as `TuiHostDispatchRedraw` then `TuiHostProcessNext` with a fixed inner `max_spins` of `64`. Stops when both steps would be idle (`0`). `max_iterations` is clamped to `0..=1_000_000`. Pushes `()`.                           |

### Pascal names (registry + compiler)

| Pascal `Std.Tui` call | Maps to intrinsic |
| ----------------------- | ----------------- |
| `Application.HostPollNext(App)` | `TuiHostPollNext` |
| `Application.HostRegisterOnKeyPressed(App, OnKeyPressed)` | `TuiHostRegisterOnKeyPressed` |
| `Application.HostInvokeOnKeyPressed(App, Key)` | `TuiHostInvokeOnKeyPressed` |
| `Application.HostRegisterOnResize(App, OnResize)` | `TuiHostRegisterOnResize` |
| `Application.HostProcessNext(App, MaxSpins)` | `TuiHostProcessNext` |
| `Application.HostRegisterOnPaint(App, OnPaint)` | `TuiHostRegisterOnPaint` |
| `Application.HostDispatchRedraw(App)` | `TuiHostDispatchRedraw` |
| `Application.HostRunLoop(App, MaxIterations)` | `TuiHostRunLoop` |

**Bytecode discriminants** (authoritative enum: [`Intrinsic`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)): **255** `TuiHostPollNext`, **256** `TuiHostRegisterOnKeyPressed`, **257** `TuiHostInvokeOnKeyPressed`, **258** `TuiHostRegisterOnResize`, **259** `TuiHostProcessNext`, **260** `TuiHostRegisterOnPaint`, **261** `TuiHostDispatchRedraw`, **262** `TuiHostRunLoop`.

`Application.Close` clears registered host handlers (`OnKeyPressed`, `OnResize`, `OnPaint`), resets the host pump state, and closes the session as today.

---

## Relationship to the poll-style API

Today, `[tui.md](tui.md)` documents `**Application.ReadEvent`**, `**ReadEventTimeout`**, `**PollEvent**`, and redraw helpers. The dispatch model replaces that pattern for full applications: the runtime owns the blocking loop and calls user `**On***` procedures. Poll-style entry points are **removed or narrowed** once dispatch mode exists (project policy: no backward compatibility requirement).

Dispatch-mode names use the `**On` prefix** so they do not collide with legacy names such as console `**KeyPressed`** (boolean poll).

---

## Session and entry


| Step                | Meaning                                                                                                                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Application.Open`  | Same session semantics as today: acquire terminal state (raw mode, alternate screen when applicable).                                                                 |
| `Application.Run`   | Start the **hosted** main loop for the given `Application` handle. **Blocks** until the host decides to stop (user quit, host error, or future signal path).          |
| `Application.Close` | Release the session. After `**Application.Run`** completes successfully, the host **must** have restored the session as if `**Close`** ran (see **Lifecycle** below). |


**VM today:** Pascal does not lower `**Application.Run`**. The closest bytecode helper is `**TuiHostRunLoop**` (**262**): a **bounded** loop that alternates redraw dispatch and `**TuiHostProcessNext`** until both are idle; it does **not** replace a blocking `**Run`** (no quit signal, `**OnExit**`, or automatic `**Close**`).

### Lifecycle (normative)

1. User calls `**Application.Open`** → receives `**App`**.
2. User calls `**Application.Run(App, …)`** with handler configuration (exact bundle syntax is Phase 4; see **Handler bundle** below).
3. While running, the host dispatches `**On*`** handlers on the **main VM thread** only (see `[parallel-vm.md](../../rust/parallel-vm.md)`).
4. When the host stops the loop, it invokes `**OnExit(App, Reason)`** once if that handler is provided, then **performs `Application.Close(App)`** (or equivalent) so the program must **not** call `**Close`** again for the same successful `**Run`** unless the spec explicitly documents a double-close error.

If `**Run`** is never called, the program keeps today’s obligation: `**Open**` / `**Close**` pairing without `**Run**`.

---

## Handler bundle (conceptual)

The compiler may lower a **single** entry (for example `**Application.Run`** plus a descriptor record) or a small sequence of registration calls. Semantically there is **one** configuration object with named handler slots.

**Required** for a minimal app: at least `**OnPaint`** (or the host rejects `**Run`**). Other slots are optional unless diagnostics require them.

Conceptual field names (logical, not final syntax):


| Slot           | Required | Role                                                                                                                            |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `OnStartup`    | no       | Runs once **before** the first blocking wait, after the session is open. Use for initial `**RequestRedraw`** or one-time setup. |
| `OnKeyPressed` | no       | Key / text input.                                                                                                               |
| `OnResize`     | no       | Terminal size changed (coalesced by the host).                                                                                  |
| `OnPaint`      | **yes**  | Full logical **frame**: draw the entire TUI for this pass.                                                                      |
| `OnIdle`       | no       | Host-invoked when no input arrived for a configured **idle interval** (optional timer; **zero** means no idle callbacks).       |
| `OnExit`       | no       | Last user hook before terminal restore (see `**OnExit`**).                                                                      |


---

## Types and signatures

Reuse existing types from `**Std.Tui`** and `**Std.Console`** where possible: `**Application**`, `**Size**`, `**Std.Console.KeyEvent**`.

### `ExitReason` (target)

Enum describing why the hosted loop stopped (exact name may be `**Std.Tui.ExitReason**`):


| Variant    | Meaning                                                                                                                                                    |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `UserQuit` | Normal exit requested by the application (for example Escape handled in `**OnKeyPressed**` calling a host **quit** primitive—Phase 3 names the intrinsic). |
| `HostStop` | Host ended the loop for an internal reason (documented per implementation).                                                                                |


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