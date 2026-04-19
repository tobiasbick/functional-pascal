# TUI application framework (Turbo Vision direction)

This document is an **implementation plan** for evolving Functional Pascal’s terminal UI from **poll-style** APIs toward a **Rust-hosted event loop** with **user-defined reactions** in FPAS (`RunApp`-style entry and `On*` callbacks). It builds on existing pieces: `[Std.Tui](../pascal/std/tui.md)`, console key types in `[Std.Console](../pascal/std/console.md)`, VM shared state and mutex discipline (see `[parallel-vm.md](../rust/parallel-vm.md#phase-3-shared-state-queues-and-io)`), and `crates/fpas-std` / `crates/fpas-vm`.

**Principles**

- **Heavy lifting in Rust** first: one coherent event loop, terminal integration, buffering, and later layout/widgets.
- **FPAS defines behavior**: register handlers; avoid duplicating a manual `repeat … ReadEventTimeout` in every program once the framework exists.
- **No backward compatibility requirement** for removed APIs (see root `AGENTS.md`); replace or shrink legacy surface deliberately.
- **Naming**: public user callbacks use the `**On` prefix** (`OnResize`, `OnKeyPressed`, …). Document which events are **best-effort** on real terminals (e.g. key up/release).

---

## Rolling progress (implementation snapshot)

Quick view of what already exists **in tree** versus what this document still treats as future work. Details stay in the phase sections below.


| Topic                                                                     | In code today                                                                                                                                    | Still open (this plan)                                                                                                      |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `**TuiHost`** resize coalescing, `**HostEvent`**, `**TuiSession`** redraw | `crates/fpas-std`                                                                                                                                | Standalone Rust-only blocking app loop (optional)                                                                           |
| VM `**TuiState**` + host intrinsics **255**–**264**                       | `crates/fpas-vm`, `crates/fpas-bytecode`; `**TuiState`** holds optional `**on_exit**` + `**last_exit_reason**`, and intrinsic **264** registers `**on_exit**` (still not invoked/populated by the run loop) | Single blocking `**Application.Run**` / intrinsic that **fills** `**last_exit_reason**`, invokes `**on_exit**`, `**Close**` |
| `**Application.Host***` + `**HostRequestQuit**` / `**HostRegisterOnExit**` | sema, compiler, VM tests, `examples/pascal/tui/host_dispatch_*.fpas`                                                                             | `**OnExit**` invocation, `**ExitReason**` from host, **idle**                                                               |
| `**Std.Tui.ExitReason**` (`**UserQuit**`, `**HostStop**`)                 | sema registry + compiler enum tables (`fpas-std` variant list)                                                                                   | Host sets `**last_exit_reason**` + Pascal `**OnExit**` when `**Run**` exists                                                |


**Next slice (in order):** (1) ~~register `**ExitReason`** in `Std.Tui~~` — **done**; (2) `~~**TuiState.on_exit**` / `**last_exit_reason**` fields (reset on `**Open**`/`**Close**`)~~ — **done**; (3) ~~`**TuiHostRegisterOnExit`** (or bundle) + intrinsic(s) to assign `**on_exit**`~~ — **done** (`**Application.HostRegisterOnExit`** / intrinsic **264** store the handler); (4) `**Application.Run**` (or one intrinsic) that runs the hosted loop, sets `**last_exit_reason**`, calls `**on_exit**`, then `**Close**` semantics; (5) `**OnIdle**` interval host-side.

---

## Phase 0 — Requirements and terminology

Phase 0 is **complete** for planning purposes. The subsections below are authoritative; revise them here if decisions change.

### Problem statement

**Goals**

- Provide a **Turbo Vision–like** application shape: one structured session replaces copy-pasted `repeat … ReadEventTimeout` loops in every program.
- Keep **terminal integration, event normalization, buffering, and timing** in **Rust** (`fpas-std` and the host), while **user reactions** live in **FPAS** as registered `On*` procedures (**dispatch** model).
- Stay consistent with the **parallel VM**: console and TUI state remain behind the mutex and ordering rules in `[parallel-vm.md](../rust/parallel-vm.md#phase-3-shared-state-queues-and-io)`.

**Non-goals (this plan)**

- **GUI-grade** input fidelity (full key-up/down and modifier parity with windowing toolkits).
- A **complete** Turbo Vision widget layer (menus, desktop, drag-and-drop); later phases may add handles and commands incrementally.
- Language-level `**async`/`await`** or algebraic effects for TUI (track separately).

### Glossary


| Term               | Meaning                                                                                                                                                                                                |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Poll**           | User code repeatedly requests input (for example `ReadEventTimeout`, `PollEvent`) and branches; the runtime offers primitives but does not own the outer application loop.                             |
| **Dispatch**       | A **Rust-hosted** loop waits on the terminal, maps input to **host events**, then calls the matching **FP handler** with prepared arguments.                                                           |
| **Session**        | The interval from acquiring the terminal (`**Application.Open`**, or a future single entry point) through `**Application.Close`** or shutdown: modes, alternate screen when used, and restore on exit. |
| **Handler**        | A user `**On*`** procedure registered for a class of host events (for example `OnResize`). Exact signatures are fixed in Phase 1.                                                                      |
| **Redraw request** | A logical dirty flag: user or host may set it; the framework coalesces and eventually runs one paint path (`**OnPaint`** — canonical name in `[tui-app.md](../pascal/std/tui-app.md)`).                |
| **Frame**          | One **logical** full paint pass after coalesced updates—not necessarily one host wake per physical terminal change.                                                                                    |


### Terminals and backends (first target)

- **Primary backend:** **crossterm** (workspace dependency, wired through `crates/fpas-std`) on **Windows, Linux, and macOS**.
- **Honest limitations:** key **release**, some **modifier** combinations, **focus**, and **paste** are **best-effort** or depend on terminal capability; the spec must mark such events **optional** where the backend cannot guarantee them.

### Threading model

- `**On*` handlers run only on the main VM thread** (the thread that runs the main task and host dispatch). Worker-pool threads must **not** enter FP bytecode for TUI callbacks unless a **later phase** explicitly defines that.
- **Spawned tasks** follow `[docs/pascal/08-concurrency.md](../pascal/08-concurrency.md)` and `[parallel-vm.md](../rust/parallel-vm.md)`: no unsynchronized sharing of the same console/TUI state as the main thread unless a future spec adds a safe API.

### Reentrancy (initial rules)

- **Keep reentrancy minimal** to avoid deadlocks and undefined ordering: during an `**On*`** handler, **do not** call into `**Std.Tui` / `Std.Console`** APIs that **block on events**, **start a nested application run**, or **open modal host dialogs** until a later phase defines semantics.
- **Non-blocking** calls such as `**RequestRedraw`** and simple session queries that do not wait on the host loop are in scope.

---

## Phase 1 — Target FPAS surface (spec before code)

Phase 1 is **complete**. The canonical user-facing spec is `**[docs/pascal/std/tui-app.md](../pascal/std/tui-app.md)`** (English). Summary:

- **Entry:** `**Application.Open`** then `**Application.Run(App, …)`** (blocking hosted loop). After `**Run`** completes normally, the host performs `**Application.Close`** semantics once `**OnExit`** has run (see `**tui-app.md`** lifecycle). Until Pascal lowers `**Run`**, bytecode can approximate stepping with `**TuiHostRunLoop**` (intrinsic **262**) plus cooperative `**TuiHostRequestQuit`** (intrinsic **263**) / Pascal `**Application.HostRequestQuit`**: a bounded loop of redraw dispatch plus `**TuiHostProcessNext`**, stopping when both are idle **or** quit was requested—not a substitute for structured `**ExitReason`** / `**OnExit`** / automatic `**Close`**.
- **Handlers:** `**OnStartup`** (optional), `**OnKeyPressed(App, Key): boolean`** (consumed), `**OnResize`**, `**OnPaint`** (required), `**OnIdle**` (optional), `**OnExit(App, Reason)**` with **no veto**.
- **Redraw:** invalidation + coalescing; `**OnPaint`** is the single FP paint contract; optional `**OnIdle`** uses a configurable idle interval.
- **Naming:** `**On*`** prefix only for dispatch callbacks; poll-style `**ReadEvent`** / `**PollEvent`** / console `**KeyPressed`** are superseded for full apps when implementation lands (`[tui.md](../pascal/std/tui.md)` remains the source for the **current** poll API until Phase 5).
- **Sema alignment:** extend `[loaded/tui.rs](../../crates/fpas-sema/src/std_registry/loaded/tui.rs)` when implementing; keep `**tui-app.md`** and that registry in sync.

---

## Phase 2 — Rust: core event loop (no FP handlers yet)

**Status:** **Largely complete** for the `**TuiHost` / `HostEvent` state machine** in `crates/fpas-std`; the **outer blocking “application loop”** is not a single Rust entry point yet—the VM drives stepping via Phase 3 intrinsics (see below).

**Done**

- `**[TuiHost](../../crates/fpas-std/src/tui_host.rs)`** and `**HostEvent`**: normalize console/`TuiSession` input; resize coalescing ahead of key; `flush_pending_resize`; `**HostEvent::suggests_request_redraw`** on resize.
- **Pump API:** `poll_next` / `read_next_blocking` over `**TuiSession`**; optional `**set_trace_hook`** for ordering/debug.
- **Redraw coordination:** `TuiSession` invalidation + `**is_redraw_pending`** (peek, consume-on-paint path used by the VM).
- **Tests:** `KeyInput` / console queues in `fpas-std` (no real terminal required for those tests).

**Open / deferred**

- A **standalone** Rust `ApplicationRuntime` that owns the **only** blocking loop for full apps (superseded for now by the VM + intrinsics story).
- **Extra `HostEvent` variants** (focus, paste, key-up) remain stubs or best-effort where documented.
- **Structured logging** beyond the trace hook is optional follow-up.

Original checklist (for history): ~~coalescing~~, ~~redraw integration~~, ~~fake-stream testing~~; **blocking Rust-only loop** and **full structured logging** still optional.

---

## Phase 3 — VM bridge: from host loop to bytecode

**Status:** **Partial** — bytecode bridge and handler dispatch are **implemented**; `**Application.Host*`** is **lowered from Pascal** (see [tui-app.md](../pascal/std/tui-app.md)). **Cooperative quit** for the bounded host loop is **implemented** (`**TuiHostRequestQuit`** / `**Application.HostRequestQuit`**). `**Application.HostRegisterOnExit`** / intrinsic **264** store an optional `**on_exit**` handler in `**TuiState**`. **Language type** `**Std.Tui.ExitReason`** is **registered** (variants `**UserQuit`** / `**HostStop**`) for future `**OnExit**` / `**Run**` — **no intrinsic assigns** `last_exit_reason` **or invokes** `on_exit` **yet**. `**TuiState`** holds optional `**on_exit**` / `**last_exit_reason**` (cleared on `**Open**`/`**Close**`). **Still missing:** `**Application.Run`**, host-produced `ExitReason` into `last_exit_reason`, `**OnExit`** call, and **idle** as a **single** hosted lifecycle.

**Done**

- `**TuiState`** (`fpas-vm`): holds `**TuiHost`** plus optional `**on_key_pressed`**, `**on_resize**`, `**on_paint**`, optional `**on_exit**` / `**last_exit_reason**` (placeholders for `**Run**`/`**OnExit**`; cleared on `**Open**`/`**Close**`) (`fpas_bytecode::Value`).
- **Intrinsics (discriminants):** **255**–**259** — `TuiHostPollNext`, register/invoke key, register resize, `TuiHostProcessNext`; **260**–**261** — `TuiHostRegisterOnPaint`, `TuiHostDispatchRedraw`; **262** — `**TuiHostRunLoop`** (bounded alternation of redraw dispatch + `ProcessNext` until idle or `**TuiHostRequestQuit`**); **263** — `**TuiHostRequestQuit`** (cooperative stop for the bounded loop); **264** — `**TuiHostRegisterOnExit`** (stores `**on_exit**` only).
- **Dispatch:** `HostEvent` → push args → `**Worker::call_function_sync`** for FP handlers; **no `tui` mutex held** across the call (see `[parallel-vm.md](../rust/parallel-vm.md)`).
- **Console helpers:** e.g. `Std.Console.KeyEvent` materialization where needed for host paths.
- **Tests:** `[tui_host_vm.rs](../../crates/fpas-vm/src/tests/core/tui_host_vm.rs)` (poll coalescing, resize/key tags, redraw tags, run loop).

**Still open**

- **One** intrinsic (or Pascal `**Application.Run`**) that blocks until stop with `**ExitReason`**, `**OnExit**`, and documented `**Close**` semantics (not the same as bounded `**TuiHostRunLoop**`).
- **Idle timer** / `**OnIdle`** wiring from the host.

**Checklist mapping**


| Item                                     | State                                                                                                                                                                                            |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1. Intrinsic set (run / register / quit) | **Partial:** **255**–**264** shipped (including cooperative quit and `**HostRegisterOnExit**` storage); `**ExitReason`** exists as a **Pascal enum only** at runtime; `**Application.Run` / runtime `ExitReason` / `OnExit` invocation** still outstanding. |
| 2. Where handlers live                   | **Done for VM:** chunk **constants** + `TuiState` fields; compiler may add a table later.                                                                                                        |
| 3. Rust → FP calling convention          | **Done** for current intrinsics (`call_function_sync`, arity checks).                                                                                                                            |
| 4. Dispatch in `fpas-vm`                 | **Done** for `ProcessNext` / redraw / run-loop stepping (not full TV lifecycle).                                                                                                                 |
| 5. Mutex / main-thread rules             | **Done** (aligned with `parallel-vm` Phase 3).                                                                                                                                                   |
| 6. VM tests with synthetic events        | **Done** (`tui_host_vm` + related).                                                                                                                                                              |


---

## Phase 4 — Compiler and semantic analysis

**Status:** **Partial.** `**Application.Host*`** symbols for the VM host intrinsics (**255**–**264**) are registered in `[loaded/tui.rs](../../crates/fpas-sema/src/std_registry/loaded/tui.rs)` and lowered in `[std_calls/tui.rs](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs)`; compiler integration tests cover stepping intrinsics, `**HostRegisterOnPaint` + `HostDispatchRedraw`**, `**HostRequestQuit` + `HostRunLoop`**, and `**HostRegisterOnExit`** storage lowering (see `fpas-compiler` `[std_library/tui.rs](../../crates/fpas-compiler/src/tests/std_library/tui.rs)`). `Std.Tui.ExitReason` is registered with the same crate (no `**Application.Run**` lowering yet). Samples: `[host_dispatch_minimal.fpas](../../examples/pascal/tui/host_dispatch_minimal.fpas)`, `[host_dispatch_paint.fpas](../../examples/pascal/tui/host_dispatch_paint.fpas)`, `[host_dispatch_quit.fpas](../../examples/pascal/tui/host_dispatch_quit.fpas)`. See [tui-app.md](../pascal/std/tui-app.md). Still open: `**Application.Run`** (or equivalent), **host wiring** for `**ExitReason`**, handler **bundle** types, and `**OnExit`** as a **single** hosted lifecycle (beyond cooperative quit on the bounded loop).

1. Extend `**Std.Tui`** (or successor unit) in `[fpas-sema` registry](../../crates/fpas-sema/src/std_registry/loaded/tui.rs): new types for **options bundle** or **fluent registration** API—keep **one** story, avoid duplicate entry points.
2. Type-check handler assignments: **procedure types** must match declared `On*` signatures exactly.
3. Lower new calls in `[fpas-compiler](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs)`: emit **registration + `Run`** sequence or a single `**Application.Run**` intrinsic with a descriptor record.
4. Update **short-name / qualified-name** rules and integration tests under `crates/fpas-sema/src/tests/integration/std_units/tui.rs`.
5. Fail with **LLM-friendly diagnostics** when handlers are missing required fields or when old APIs are used after removal.

---

## Phase 5 — Standard library cleanup: TUI-first, shrink legacy console loop

**Status:** **Not started** (depends on Phase 4).

1. Inventory **poll-style** entry points: `ReadEvent`, `ReadEventTimeout`, `PollEvent`, `KeyPressed`, etc., across `[docs/pascal/std/console.md](../pascal/std/console.md)` and `[tui.md](../pascal/std/tui.md)`.
2. Classify each as: **keep for non-TUI scripts**, **move under `Std.Tui`**, or **remove** in favor of `On*`.
3. Update **Mandelbrot** and other examples to the **new** pattern once `Run` exists; delete duplicated loop boilerplate.
4. Refresh `**[examples/README.md](../../examples/README.md)`** and any **CI** that assumes old patterns.

---

## Phase 6 — Event coverage and honesty in the spec

**Status:** **Not started** (partial overlap with Phase 4 once dispatch ships).

1. Implement `**OnKeyPressed`** (and `**OnKeyReleased` / `OnKeyDown`** only if the backend can emit them reliably; otherwise document as **optional / noop** on some platforms).
2. Implement `**OnResize`** with **debounced** size (match `Application.Size` semantics).
3. Implement `**OnExit`** (or `**OnShutdown`**) for clean terminal restore; pair with `**Application.Open`/`Close`** lifecycle.
4. Add `**OnPaint**` tied to **invalidation** and `RedrawPending` semantics (see `[tui-app.md](../pascal/std/tui-app.md)`).
5. Document **every** `On`* in `docs/pascal/std/` with **when it fires** and **threading** expectations.

---

## Phase 7 — Toward Turbo Vision–like structure (incremental)

**Status:** **Not started**.

1. Introduce **view IDs** or **handles** in Rust only: opaque to FPAS at first (`type View = …` with no methods).
2. Add **child ordering** and **focus chain** in Rust; expose **minimal** FP callbacks: `OnActivate`, `OnDeactivate` (names TBD).
3. Add **command set** (keyboard shortcuts) resolved in Rust, **invoking** FP handlers by id—avoid parsing key names in FP for common cases.
4. Add **modal dialog** host API once single-view dispatch is stable.
5. Revisit **performance**: double buffer, **damage rectangles**—likely Rust-only for a long time.

---

## Phase 8 — Quality, tooling, and maintenance

**Status:** **Not started** (incremental work alongside Phases 4–7).

1. **Integration test**: headless or scripted terminal where possible; document manual test checklist for real terminals.
2. **Fuzz or property-test** event ordering (resize bursts, rapid keys).
3. **Performance budget**: max latency from input to handler on a reference machine (informal).
4. **Link from Rust sources** to the canonical `docs/pascal/` spec for any implemented behavior (project rule).
5. Update `**[docs/future/README.md](README.md)`** when milestones complete; consider moving this file to a “completed” subsection or archiving phases.

---

## Dependency graph (summary)


| Phase           | Depends on   |
| --------------- | ------------ |
| 1 Spec          | 0            |
| 2 Rust loop     | 0–1          |
| 3 VM bridge     | 2            |
| 4 Compiler      | 3            |
| 5 Cleanup       | 4 + examples |
| 6 Event honesty | 4            |
| 7 TV-like       | 6            |
| 8 QA            | 5–7          |


Phases **6** and **7** can partially overlap once **4** is stable; do not start **7** before **dispatch and redraw** stories are trustworthy.

---

## Out of scope for this plan (track separately)

- **Algebraic effects** or language-level `async`/`await` for TUI (possible future language work).
- **Full** Turbo Vision widget set (menus, desktop, drag-drop)—this plan only **anchors** the host and callback architecture.

---

## Related documentation


| Document                                                       | Relevance                                                                                           |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `[docs/pascal/std/tui.md](../pascal/std/tui.md)`               | Current `Std.Tui` API (poll-style)                                                                  |
| `[docs/pascal/std/tui-app.md](../pascal/std/tui-app.md)`       | Target dispatch-mode API (`Application.Run`, `On`*)                                                 |
| `[docs/pascal/std/console.md](../pascal/std/console.md)`       | Key types and legacy I/O                                                                            |
| `[docs/rust/parallel-vm.md](../rust/parallel-vm.md)`           | Implemented VM task runtime; Phase 3 — shared I/O and locks (`#phase-3-shared-state-queues-and-io`) |
| `[docs/pascal/08-concurrency.md](../pascal/08-concurrency.md)` | Task model vs TUI main thread                                                                       |


