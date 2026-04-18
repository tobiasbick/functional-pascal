# TUI application framework (Turbo Vision direction)

This document is an **implementation plan** for evolving Functional Pascal’s terminal UI from **poll-style** APIs toward a **Rust-hosted event loop** with **user-defined reactions** in FPAS (`RunApp`-style entry and `On*` callbacks). It builds on existing pieces: `[Std.Tui](../pascal/std/tui.md)`, console key types in `[Std.Console](../pascal/std/console.md)`, VM shared state and mutex discipline (see `[parallel-vm.md](../rust/parallel-vm.md#phase-3-shared-state-queues-and-io)`), and `crates/fpas-std` / `crates/fpas-vm`.

**Principles**

- **Heavy lifting in Rust** first: one coherent event loop, terminal integration, buffering, and later layout/widgets.
- **FPAS defines behavior**: register handlers; avoid duplicating a manual `repeat … ReadEventTimeout` in every program once the framework exists.
- **No backward compatibility requirement** for removed APIs (see root `AGENTS.md`); replace or shrink legacy surface deliberately.
- **Naming**: public user callbacks use the `**On` prefix** (`OnResize`, `OnKeyPressed`, …). Document which events are **best-effort** on real terminals (e.g. key up/release).

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
| **Session**        | The interval from acquiring the terminal (`**Application.Open`**, or a future single entry point) through `**Application.Close**` or shutdown: modes, alternate screen when used, and restore on exit. |
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

- **Entry:** `**Application.Open`** then `**Application.Run(App, …)**` (blocking hosted loop). After `**Run**` completes normally, the host performs `**Application.Close**` semantics once `**OnExit**` has run (see `**tui-app.md**` lifecycle).
- **Handlers:** `**OnStartup`** (optional), `**OnKeyPressed(App, Key): boolean**` (consumed), `**OnResize**`, `**OnPaint**` (required), `**OnIdle**` (optional), `**OnExit(App, Reason)**` with **no veto**.
- **Redraw:** invalidation + coalescing; `**OnPaint`** is the single FP paint contract; optional `**OnIdle**` uses a configurable idle interval.
- **Naming:** `**On*`** prefix only for dispatch callbacks; poll-style `**ReadEvent**` / `**PollEvent**` / console `**KeyPressed**` are superseded for full apps when implementation lands (`[tui.md](../pascal/std/tui.md)` remains the source for the **current** poll API until Phase 5).
- **Sema alignment:** extend `[loaded/tui.rs](../../crates/fpas-sema/src/std_registry/loaded/tui.rs)` when implementing; keep `**tui-app.md`** and that registry in sync.

---

## Phase 2 — Rust: core event loop (no FP handlers yet)

1. Introduce a `**TuiHost` / `ApplicationRuntime**` module in `crates/fpas-std` (or a dedicated subcrate if boundaries blur) that owns the **blocking loop**: read terminal events, normalize to an internal `**HostEvent`** enum.
2. Map low-level events to `**HostEvent**`: at minimum **resize**, **key down / text**, **focus gained/lost** if already used, **paste** if in scope; leave stubs for **key up** if the backend cannot supply them.
3. Implement **coalescing** where needed (e.g. multiple resize notifications → one logical `OnResize`).
4. Integrate **redraw requests**: internal flag set by `RequestRedraw` and/or host-driven “needs paint” after certain events.
5. Add **structured logging hooks** (behind `cfg`) for debugging dispatch order.
6. **Unit-test** the Rust state machine with **fake event streams** (no terminal required in CI).

---

## Phase 3 — VM bridge: from host loop to bytecode

1. Design **intrinsic set** for: **enter run loop**, **register N typed handlers**, **run until quit**. Prefer **few** intrinsics with clear semantics over many micro-ops.
2. Define how the VM **stores** function references: indices into a **handler table** emitted by the compiler, or chunk constants; align with existing **first-class function** representation in bytecode.
3. Specify **calling convention** from Rust into FP: **which stack**, **which arguments**, **error handling** if the user panics or returns.
4. Implement **dispatch** in `crates/fpas-vm`: on `HostEvent`, push arguments, call the registered procedure, then resume the host loop.
5. Ensure **console mutex / shared state** rules from `[parallel-vm.md](../rust/parallel-vm.md#phase-3-shared-state-queues-and-io)` still hold: no concurrent VM access from multiple threads during handler execution.
6. Add **VM tests** that register stub handlers and feed synthetic events through the bridge.

---

## Phase 4 — Compiler and semantic analysis

1. Extend `**Std.Tui`** (or successor unit) in `[fpas-sema` registry](../../crates/fpas-sema/src/std_registry/loaded/tui.rs): new types for **options bundle** or **fluent registration** API—keep **one** story, avoid duplicate entry points.
2. Type-check handler assignments: **procedure types** must match declared `On*` signatures exactly.
3. Lower new calls in `[fpas-compiler](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs)`: emit **registration + `Run`** sequence or a single `**Application.Run**` intrinsic with a descriptor record.
4. Update **short-name / qualified-name** rules and integration tests under `crates/fpas-sema/src/tests/integration/std_units/tui.rs`.
5. Fail with **LLM-friendly diagnostics** when handlers are missing required fields or when old APIs are used after removal.

---

## Phase 5 — Standard library cleanup: TUI-first, shrink legacy console loop

1. Inventory **poll-style** entry points: `ReadEvent`, `ReadEventTimeout`, `PollEvent`, `KeyPressed`, etc., across `[docs/pascal/std/console.md](../pascal/std/console.md)` and `[tui.md](../pascal/std/tui.md)`.
2. Classify each as: **keep for non-TUI scripts**, **move under `Std.Tui`**, or **remove** in favor of `On*`.
3. Update **Mandelbrot** and other examples to the **new** pattern once `Run` exists; delete duplicated loop boilerplate.
4. Refresh `**[examples/README.md](../../examples/README.md)`** and any **CI** that assumes old patterns.

---

## Phase 6 — Event coverage and honesty in the spec

1. Implement `**OnKeyPressed`** (and `**OnKeyReleased` / `OnKeyDown**` only if the backend can emit them reliably; otherwise document as **optional / noop** on some platforms).
2. Implement `**OnResize`** with **debounced** size (match `Application.Size` semantics).
3. Implement `**OnExit`** (or `**OnShutdown**`) for clean terminal restore; pair with `**Application.Open`/`Close**` lifecycle.
4. Add `**OnPaint**` tied to **invalidation** and `RedrawPending` semantics (see `[tui-app.md](../pascal/std/tui-app.md)`).
5. Document **every** `On*` in `docs/pascal/std/` with **when it fires** and **threading** expectations.

---

## Phase 7 — Toward Turbo Vision–like structure (incremental)

1. Introduce **view IDs** or **handles** in Rust only: opaque to FPAS at first (`type View = …` with no methods).
2. Add **child ordering** and **focus chain** in Rust; expose **minimal** FP callbacks: `OnActivate`, `OnDeactivate` (names TBD).
3. Add **command set** (keyboard shortcuts) resolved in Rust, **invoking** FP handlers by id—avoid parsing key names in FP for common cases.
4. Add **modal dialog** host API once single-view dispatch is stable.
5. Revisit **performance**: double buffer, **damage rectangles**—likely Rust-only for a long time.

---

## Phase 8 — Quality, tooling, and maintenance

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


