# TUI application framework (Turbo Vision direction)

Implementation plan for evolving Functional Pascal's terminal UI from poll-style APIs toward a Rust-hosted event loop with `On*` dispatch callbacks. Canonical user-facing spec: `[docs/pascal/std/tui-app.md](../pascal/std/tui-app.md)`.

**Principles**

- **Heavy lifting in Rust**: coherent event loop, terminal integration, buffering, and later layout/widgets.
- **FPAS defines behavior**: register `On`* handlers; no manual `repeat … ReadEventTimeout` loops in user programs.
- **No backward compatibility** for removed APIs (see root `AGENTS.md`).

---

## Completed phases (0–6)


| Phase                 | What was done                                                                                                                                                                                                                                                                                                               | Key artifacts                                                                                                                                            |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **0** Requirements    | Goals, glossary, threading model, reentrancy rules, crossterm backend choice.                                                                                                                                                                                                                                               | This file (authoritative)                                                                                                                                |
| **1** FPAS spec       | Handler signatures, `ExitReason`, `ApplicationHandlers` record, `Application.Run` lifecycle.                                                                                                                                                                                                                                | `[tui-app.md](../pascal/std/tui-app.md)`                                                                                                                 |
| **2** Rust event loop | `TuiHost` + `HostEvent` normalization, resize coalescing, `poll_next` / `read_next_blocking`, `set_trace_hook`, `TuiSession` redraw coordination. Standalone `ApplicationRuntime` superseded by VM intrinsics.                                                                                                              | `[fpas-std/src/tui_host.rs](../../crates/fpas-std/src/tui_host.rs)`                                                                                      |
| **3** VM bridge       | Intrinsics **255**–**266**: poll, register/invoke handlers, `ProcessNext`, `DispatchRedraw`, bounded `RunLoop`, cooperative `RequestQuit`, `RegisterOnExit`, `Application.Run` (hosted loop + `OnIdle` + `OnExit` + auto-close), `RegisterOnIdle`. `ExitReason`: `UserQuit`, `HostStop`, `HostAndUserStop`, `HostShutdown`. | `[tui_run.rs](../../crates/fpas-vm/src/vm/execute/io/tui_run.rs)`, `[tui_host_vm.rs](../../crates/fpas-vm/src/tests/core/tui_host_vm.rs)`                |
| **4** Compiler + sema | Intrinsics **255**–**271** registered and lowered; `ApplicationHandlers` record + `Application.Configure` (intrinsic **267**) with type-checking and LLM-friendly diagnostics.                                                                                                                                              | `[loaded/tui.rs](../../crates/fpas-sema/src/std_registry/loaded/tui.rs)`, `[std_calls/tui.rs](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs)` |
| **5** Std cleanup     | Poll-style API classified in `tui.md`; `minimal_application.fpas` rewritten to dispatch model; `OnKeyPressed` + `OnResize` integration tests.                                                                                                                                                                               | `[tui.md](../pascal/std/tui.md)`, `[tui_configure.rs](../../crates/fpas-compiler/src/tests/std_library/tui_configure.rs)`                                |
| **6** Event coverage  | `OnMouse` (intrinsic **268**), `OnPaste` (**269**), `OnFocusGained` (**270**), `OnFocusLost` (**271**) — variants in `TuiEvent`/`HostEvent`, bundle fields, dispatch tags, tests. All `On`* documented in `tui-app.md` with firing rules and threading.                                                                     | `[tui-app.md](../pascal/std/tui-app.md)`                                                                                                                 |


---

## Phase 7 — Toward Turbo Vision–like structure (incremental)

**Status:** Steps 1–7 are done for the current hosted-dispatch scope. In short: hosted dispatch works, view handles/focus chain/commands/modals are in place, modal-scoped routing exists, dirty-rectangle production now covers the current host-managed events, and hosted paint runs through a deferred back-buffered present.

### Done so far

- Hosted TUI loop, `On*` handler registration, `Application.Configure`, `Application.Run`, and `ExitReason` are in place.
- Host-managed views now cover registration, unregister, focus traversal, modal attachment, and `Application.HostSetViewRect`.
- Damage tracking exists end to end and now uses explicit dirty rectangles or redraw hints for focus transitions, view lifecycle changes, view-rect updates, resize, modal attach/leave, mouse, paste, and terminal focus events.
- Hosted `OnPaint` now runs through a deferred single-present path: CRT writes stay buffered until the handler returns, and the terminal diff/flush step can restrict itself to the tracked dirty region plus the actual console mutations recorded during that frame.

### What is still missing

- `OnPaint` is still application-global. There is still no widget-local or modal-local paint contract.
- Modal scoping is still explicit and narrow: FPAS must attach views manually, and command bindings are still global rather than per-view or per-modal.
- There is still no widget tree, layout system, z-order policy beyond registration order, or high-level dialog API such as `ShowModal`.

## Next open implementation item

The next unfinished Phase 7 work item is **structure above the hosted core**:

- narrow the paint contract from application-global `OnPaint` toward view-local or modal-local paint
- move beyond explicit view attachment toward richer modal/view ownership and routing rules
- add the next structural layer: widget tree, layout, z-order policy, and higher-level dialog primitives

---

## Phase 8 — Quality, tooling, and maintenance

**Status:** Not started (runs alongside Phases 7+).

1. **Integration test**: headless or scripted terminal; document manual checklist for real terminals.
2. **Fuzz / property-test** event ordering (resize bursts, rapid keys).
3. **Performance budget**: max input-to-handler latency on a reference machine (informal target).
4. Verify all Rust sources link to the canonical `docs/pascal/` spec (project rule).
5. Archive or move this file once Phase 7 is complete.

---

## Out of scope

- Algebraic effects / `async`/`await` for TUI — separate language track.
- Full Turbo Vision widget set (menus, desktop, drag-drop) — this plan only anchors the host + callback architecture.

---

## Related documentation


| Document                                                       | Relevance                                                  |
| -------------------------------------------------------------- | ---------------------------------------------------------- |
| `[docs/pascal/std/tui-app.md](../pascal/std/tui-app.md)`       | Dispatch-mode API (`Application.Run`, `On`*, `ExitReason`) |
| `[docs/pascal/std/tui.md](../pascal/std/tui.md)`               | Poll-style API status and superseded surface               |
| `[docs/pascal/std/console.md](../pascal/std/console.md)`       | Key types and legacy I/O                                   |
| `[docs/rust/parallel-vm.md](../rust/parallel-vm.md)`           | VM task runtime; shared I/O and mutex ordering             |
| `[docs/pascal/08-concurrency.md](../pascal/08-concurrency.md)` | Task model vs TUI main thread                              |
