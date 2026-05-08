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

**Status:** Steps 1–6 done. Performance groundwork has started.

1. ✅ Introduce **view IDs / handles** in Rust only: `ViewId` (opaque `u32` wrapper), `ViewRect` (bounding box), `ViewRegistry` (register / unregister / rect / clear). `TuiState.views: ViewRegistry` added; cleared on `Application.Close`. No FPAS surface. Key artifacts: [`fpas-std/src/tui_view.rs`](../../crates/fpas-std/src/tui_view.rs), [`shared.rs`](../../crates/fpas-vm/src/vm/shared.rs).
2. ✅ **Child ordering and focus chain** in Rust: `ViewRegistry` extended with an ordered focus chain (`push_child`, `remove_child`, `focus_next`, `focus_prev`, `focused_id`, `has_focusable_children`). Tab / Shift+Tab are intercepted by `tui_host_process_next_inner` and advance / retreat focus when the chain is non-empty; the key falls through to `OnKeyPressed` when there are no focusable children. `OnActivate` (intrinsic **272**) and `OnDeactivate` (intrinsic **273**) — both `procedure (Application)` — are fired by the host on every focus transition; registered via `Application.HostRegisterOnActivate` / `Application.HostRegisterOnDeactivate` or as optional fields in `ApplicationHandlers`. Focus changes also request a redraw (tags **14** = forward, **15** = backward). Key artifacts: [`tui_view.rs`](../../crates/fpas-std/src/tui_view.rs), [`run_loop.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/run_loop.rs), [`tui_focus_vm.rs`](../../crates/fpas-vm/src/tests/core/tui_focus_vm.rs).
3. ✅ Add **command set** (keyboard shortcuts) resolved in Rust, invoking FP handlers by id: `CommandRegistry` stores `Std.Console.KeyEvent` → integer command id bindings; `Application.HostBindCommand` registers shortcuts, `Application.HostRegisterOnCommand` / `ApplicationHandlers.OnCommand` register `procedure (Application, integer)`, and `HostProcessNext` dispatches commands before ordinary `OnKeyPressed` (tags **16** = dispatched, **17** = bound but no handler). Key artifacts: [`tui_command.rs`](../../crates/fpas-std/src/tui_command.rs), [`run_loop.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/run_loop.rs), [`tui_commands.rs`](../../crates/fpas-compiler/src/tests/std_library/tui_commands.rs).
4. ✅ Add **modal dialog** host API: `ModalStack` stores application-defined integer modal ids, `Application.HostEnterModal` / `Application.HostLeaveModal` mutate the stack, and `Application.HostModalDepth` exposes the active stack depth for tests and later routing. `Application.Open` / `Application.Close` clear modal state. Key artifacts: [`tui_modal.rs`](../../crates/fpas-std/src/tui_modal.rs), [`tui_modal.rs`](../../crates/fpas-compiler/src/tests/std_library/tui_modal.rs).
5. ✅ Expose a minimal **view host API** to FPAS: `Application.HostRegisterView` / `Application.HostUnregisterView` manage opaque integer view handles backed by `ViewRegistry`; `Application.HostPushChildView` seeds the Tab / Shift+Tab focus chain from FPAS; `Application.HostQueryFocusedViewId` exposes the currently focused host view id (or `-1`). This is intentionally narrower than a widget API and provides the missing bridge between Rust-managed view state and future modal/view routing. Key artifacts: [`tui_view.rs`](../../crates/fpas-std/src/tui_view.rs), [`tui/mod.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/mod.rs), [`tui_focus.rs`](../../crates/fpas-compiler/src/tests/std_library/tui_focus.rs).
6. ✅ Add **modal-scoped routing** on top of those view handles: `Application.HostAttachViewToActiveModal` associates views with the topmost modal frame; Tab / Shift+Tab traversal is limited to the modal view set while active; mouse events outside the active modal view rectangles are suppressed; key and command dispatch are blocked while focus sits on a background view outside the active modal scope. `OnPaint`, resize, paste, and terminal focus events remain global for now. Key artifacts: [`tui_modal.rs`](../../crates/fpas-std/src/tui_modal.rs), [`run_loop.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/run_loop.rs), [`tui_modal.rs`](../../crates/fpas-compiler/src/tests/std_library/tui_modal.rs).
7. Revisit **performance**: double buffer, damage rectangles — Rust-internal, no FP contract change yet.
	- ✅ Groundwork: `TuiSession` now uses an internal damage accumulator instead of a bare redraw flag. `Application.RequestRedraw` still marks the whole frame dirty for now. Key artifacts: [`fpas-std/src/tui.rs`](../../crates/fpas-std/src/tui.rs), [`fpas-std/src/tui_damage.rs`](../../crates/fpas-std/src/tui_damage.rs).
	- ✅ Hosted consumption: the VM redraw path now peeks and consumes `DamageRegion` values end to end before `OnPaint`, while preserving the current application-global paint contract. Hosted `Application.Run` callbacks also remain shutdown-safe in this path, so `HostShutdown` still reaches `OnExit` deterministically. Key artifacts: [`run_loop.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/run_loop.rs), [`sync_call.rs`](../../crates/fpas-vm/src/vm/execute/io/callbacks/sync_call.rs), [`tui_run.rs`](../../crates/fpas-vm/src/vm/execute/io/tui_run.rs).

### Current limitations after Step 6

- `OnPaint` remains application-global. The host still expects a full-frame paint contract; there is no modal-only paint surface and no widget-local redraw protocol yet.
- Resize, paste, terminal focus-gained, and terminal focus-lost events remain application-global. Current modal routing only constrains Tab / Shift+Tab traversal, mouse hits, and key / command dispatch.
- Modal scoping is explicit and narrow: FPAS must call `Application.HostAttachViewToActiveModal(App, ViewId)` for each modal view. A modal frame without attached views still behaves like state only.
- Command bindings remain global registrations. The host currently blocks command dispatch when focus is outside the active modal scope, but there is no per-view or per-modal command registry yet.
- Damage tracking is only partially exploited. Today every `Application.RequestRedraw` still invalidates the whole frame, and no host subsystem emits partial dirty rectangles yet.
- There is still no widget tree, layout system, z-order policy beyond registration order, or high-level dialog abstraction such as `ShowModal`.

## Next open implementation item

The next unfinished Phase 7 work item is still **performance**, but the immediate next code step is now narrower:

- wire partial dirty-rectangle producers into the Rust host instead of always marking full-frame redraws
- add double buffering after dirty-rectangle flow exists end to end
- keep the FPAS contract unchanged unless measurement forces a later spec adjustment

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
