# TUI application framework (Turbo Vision direction)

This document is an **implementation plan** for evolving Functional Pascal’s terminal UI from **poll-style** APIs toward a **Rust-hosted event loop** with **user-defined reactions** in FPAS (`RunApp`-style entry and `On*` callbacks). It builds on existing pieces: [`Std.Tui`](../pascal/std/tui.md), console key types in [`Std.Console`](../pascal/std/console.md), VM shared state and mutex discipline (see [`parallel-vm.md`](parallel-vm.md#phase-3-shared-state-queues-and-io)), and `crates/fpas-std` / `crates/fpas-vm`.

**Principles**

- **Heavy lifting in Rust** first: one coherent event loop, terminal integration, buffering, and later layout/widgets.
- **FPAS defines behavior**: register handlers; avoid duplicating a manual `repeat … ReadEventTimeout` in every program once the framework exists.
- **No backward compatibility requirement** for removed APIs (see root `AGENTS.md`); replace or shrink legacy surface deliberately.
- **Naming**: public user callbacks use the **`On` prefix** (`OnResize`, `OnKeyPressed`, …). Document which events are **best-effort** on real terminals (e.g. key up/release).

---

## Phase 0 — Requirements and terminology

1. Write a one-page **problem statement**: goals (Turbo Vision–like structure, Rust-first), non-goals (e.g. full GUI parity for key lifecycle).
2. Fix **glossary**: *poll* vs *dispatch*, *session*, *handler*, *redraw request*, *frame*.
3. List **terminals and backends** you intend to support first (e.g. crossterm on Windows/Linux/macOS) and note **limitations** (key release, modifiers).
4. Decide **threading model**: handlers run on the **main VM thread** only unless a later phase explicitly allows worker callbacks (default: **single-threaded dispatch** into FP bytecode).
5. Decide **reentrancy rules**: whether handlers may call back into `Std.Tui` / `Std.Console` APIs that enqueue work or open dialogs (initial answer should be **minimal** to avoid deadlocks).

---

## Phase 1 — Target FPAS surface (spec before code)

1. Draft the **user-visible API** in Pascal (not necessarily final syntax): e.g. `Application.Run` or `RunApp`, and registration of `OnKeyPressed`, `OnResize`, `OnExit`, `OnPaint` or `OnRedraw`, etc.
2. For each callback, define **signature**: parameters (e.g. `Application`, `KeyEvent`, `Size`), return type (`procedure` vs `function` returning e.g. `boolean` for “consume event”).
3. Define **`OnExit`**: when it runs (user quit vs signal), and whether it may veto shutdown (start with **no veto** for simplicity).
4. Define **redraw model**: push `RequestRedraw` from Rust vs user-only `OnPaint` every frame; prefer **invalidation + optional idle timer** to match Turbo Vision–style **damage** later.
5. Resolve **naming collision** with existing `KeyPressed(): boolean` (poll API): new names are **`OnKeyPressed`** etc.; old names are **removed or deprecated** per project policy.
6. Capture the spec in **`docs/pascal/std/tui.md`** (or a new `docs/pascal/std/tui-app.md` if the file grows too large) when implementation starts—**English only**, aligned with [`loaded/tui.rs`](../../crates/fpas-sema/src/std_registry/loaded/tui.rs).

---

## Phase 2 — Rust: core event loop (no FP handlers yet)

1. Introduce a **`TuiHost` / `ApplicationRuntime`** module in `crates/fpas-std` (or a dedicated subcrate if boundaries blur) that owns the **blocking loop**: read terminal events, normalize to an internal **`HostEvent`** enum.
2. Map low-level events to **`HostEvent`**: at minimum **resize**, **key down / text**, **focus gained/lost** if already used, **paste** if in scope; leave stubs for **key up** if the backend cannot supply them.
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
5. Ensure **console mutex / shared state** rules from [`parallel-vm.md`](parallel-vm.md#phase-3-shared-state-queues-and-io) still hold: no concurrent VM access from multiple threads during handler execution.
6. Add **VM tests** that register stub handlers and feed synthetic events through the bridge.

---

## Phase 4 — Compiler and semantic analysis

1. Extend **`Std.Tui`** (or successor unit) in [`fpas-sema` registry](../../crates/fpas-sema/src/std_registry/loaded/tui.rs): new types for **options bundle** or **fluent registration** API—keep **one** story, avoid duplicate entry points.
2. Type-check handler assignments: **procedure types** must match declared `On*` signatures exactly.
3. Lower new calls in [`fpas-compiler`](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs): emit **registration + `Run`** sequence or a single **`Application.Run`** intrinsic with a descriptor record.
4. Update **short-name / qualified-name** rules and integration tests under `crates/fpas-sema/src/tests/integration/std_units/tui.rs`.
5. Fail with **LLM-friendly diagnostics** when handlers are missing required fields or when old APIs are used after removal.

---

## Phase 5 — Standard library cleanup: TUI-first, shrink legacy console loop

1. Inventory **poll-style** entry points: `ReadEvent`, `ReadEventTimeout`, `PollEvent`, `KeyPressed`, etc., across [`docs/pascal/std/console.md`](../pascal/std/console.md) and [`tui.md`](../pascal/std/tui.md).
2. Classify each as: **keep for non-TUI scripts**, **move under `Std.Tui`**, or **remove** in favor of `On*`.
3. Update **Mandelbrot** and other examples to the **new** pattern once `Run` exists; delete duplicated loop boilerplate.
4. Refresh **[`examples/README.md`](../../examples/README.md)** and any **CI** that assumes old patterns.

---

## Phase 6 — Event coverage and honesty in the spec

1. Implement **`OnKeyPressed`** (and **`OnKeyReleased` / `OnKeyDown`** only if the backend can emit them reliably; otherwise document as **optional / noop** on some platforms).
2. Implement **`OnResize`** with **debounced** size (match `Application.Size` semantics).
3. Implement **`OnExit`** (or **`OnShutdown`**) for clean terminal restore; pair with **`Application.Open`/`Close`** lifecycle.
4. Add **`OnPaint`/`OnRedraw`** (name per Phase 1 decision) tied to **invalidation** and `RedrawPending` semantics.
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
5. Update **[`docs/future/README.md`](README.md)** when milestones complete; consider moving this file to a “completed” subsection or archiving phases.

---

## Dependency graph (summary)

| Phase | Depends on |
|-------|------------|
| 1 Spec | 0 |
| 2 Rust loop | 0–1 |
| 3 VM bridge | 2 |
| 4 Compiler | 3 |
| 5 Cleanup | 4 + examples |
| 6 Event honesty | 4 |
| 7 TV-like | 6 |
| 8 QA | 5–7 |

Phases **6** and **7** can partially overlap once **4** is stable; do not start **7** before **dispatch and redraw** stories are trustworthy.

---

## Out of scope for this plan (track separately)

- **Algebraic effects** or language-level `async`/`await` for TUI (possible future language work).
- **Full** Turbo Vision widget set (menus, desktop, drag-drop)—this plan only **anchors** the host and callback architecture.

---

## Related documentation

| Document | Relevance |
|----------|-----------|
| [`docs/pascal/std/tui.md`](../pascal/std/tui.md) | Current `Std.Tui` API |
| [`docs/pascal/std/console.md`](../pascal/std/console.md) | Key types and legacy I/O |
| [`docs/future/parallel-vm.md`](parallel-vm.md) | VM task roadmap; Phase 3 — shared I/O and locks |
| [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md) | Task model vs TUI main thread |
