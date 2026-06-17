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
| **2** Rust event loop | `TuiHost` + shared `UiEvent` normalization, resize coalescing, `poll_next` / `read_next_blocking`, `set_trace_hook`, `TuiSession` redraw coordination. Standalone `ApplicationRuntime` superseded by VM intrinsics.                                                                                                       | `[fpas-std/src/tui_host.rs](../../crates/fpas-std/src/tui_host.rs)`                                                                                      |
| **3** VM bridge       | Intrinsics **255**–**266**: poll, register/invoke handlers, `ProcessNext`, `DispatchRedraw`, bounded `RunLoop`, cooperative `RequestQuit`, `RegisterOnExit`, `Application.Run` (hosted loop + `OnIdle` + `OnExit` + auto-close), `RegisterOnIdle`. `ExitReason`: `UserQuit`, `HostStop`, `HostAndUserStop`, `HostShutdown`. | `[tui_run.rs](../../crates/fpas-vm/src/vm/execute/io/tui_run.rs)`, `[tui_host_vm.rs](../../crates/fpas-vm/src/tests/core/tui_host_vm.rs)`                |
| **4** Compiler + sema | Intrinsics **255**–**271** registered and lowered; `ApplicationHandlers` record + `Application.Configure` (intrinsic **267**) with type-checking and LLM-friendly diagnostics.                                                                                                                                              | `[loaded/tui.rs](../../crates/fpas-sema/src/std_registry/loaded/tui.rs)`, `[std_calls/tui.rs](../../crates/fpas-compiler/src/compiler/std_calls/tui.rs)` |
| **5** Std cleanup     | Poll-style API classified in `tui.md`; `minimal_application.fpas` rewritten to dispatch model; `OnKeyPressed` + `OnResize` integration tests.                                                                                                                                                                               | `[tui.md](../pascal/std/tui.md)`, `[tui_configure.rs](../../crates/fpas-compiler/src/tests/std_library/tui_configure.rs)`                                |
| **6** Event coverage  | `OnMouse` (intrinsic **268**), `OnPaste` (**269**), `OnFocusGained` (**270**), `OnFocusLost` (**271**) — variants in the public `TuiEvent` surface and the internal shared `UiEvent` model, plus bundle fields, dispatch tags, and tests. All `On`* documented in `tui-app.md` with firing rules and threading.          | `[tui-app.md](../pascal/std/tui-app.md)`                                                                                                                 |


---

## Phase 7 — Toward Turbo Vision–like structure (incremental)

**Status:** Complete for the current hosted-dispatch scope. Hosted dispatch works, view handles/focus chain/commands/modals are in place, modal-scoped routing exists, dirty-rectangle production covers the current host-managed events, hosted paint runs through a deferred back-buffered present, and the missing structural layer from this phase is now implemented.

### Done so far

- Hosted TUI loop, `On*` handler registration, `Application.Configure`, `Application.Run`, and `ExitReason` are in place.
- Host-managed views now cover registration, unregister, focus traversal, modal attachment, `Application.HostSetViewRect`, and `Application.HostSetViewParent`.
- The host now maintains a real view tree: child views use parent-relative layout, sibling order defines z-order, and modal scope can target a full rooted subtree instead of only an explicit flat attachment list.
- Local paint now exists alongside application-global `OnPaint`: `Application.HostRegisterOnViewPaint` installs per-view handlers that receive `Std.Tui.Rect` and run in tree paint order for damaged views.
- High-level modal structure now exists through `Application.ShowModal` / `Application.CloseModal`, and `Application.ShowDialog` now adds a minimal owned dialog primitive on top of that root-view model.
- Command maps are no longer only global: the host now resolves shortcuts from focused-view ancestry, then the active modal frame, then the global command registry.
- Damage tracking exists end to end and now uses explicit dirty rectangles or redraw hints for focus transitions, view lifecycle changes, view-rect updates, resize, modal attach/leave, mouse, paste, and terminal focus events.
- Hosted `OnPaint` now runs through a deferred single-present path: CRT writes stay buffered until the handler returns, and the terminal diff/flush step can restrict itself to the tracked dirty region plus the actual console mutations recorded during that frame.

### Deferred beyond Phase 7

- A higher-level widget library, automatic layout manager, and packaged Turbo Vision-style control set remain outside the completed Phase 7 scope.
- `ApplicationHandlers` intentionally remains the application-global handler bundle; view-local paint and scoped command maps stay on the explicit host-view / modal surfaces in the current design.

---

## Phase 8 — Quality, tooling, and maintenance

**Status:** Next active phase.

1. **Integration test**: headless or scripted terminal; manual real-terminal checklist is documented in [`docs/pascal/std/tui-terminal-checklist.md`](../pascal/std/tui-terminal-checklist.md).
2. **Fuzz / property-test** event ordering (resize bursts, rapid keys).
3. Verify TUI Rust sources link to the canonical `docs/pascal/` spec (covered by `tui_rust_sources_link_to_pascal_spec_docs`).
4. Archive or move this file now that Phase 7 is complete.

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
| `[docs/pascal/08-concurrency.md](../pascal/08-concurrency.md)` | Task model; TUI `On*` handlers run on the main thread      |
