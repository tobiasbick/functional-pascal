# Native TUI testing in FPAS — implementation plan

**Status:** Phase 3 complete; Phase 4 next.
**Design:** [`README.md`](README.md).

Trackable, resumable plan. Each task has a checkbox, concrete file anchors, and a verification step. After a context loss, **start by reading the "Resume here" marker** below, then continue at the first unchecked task.

## How to use this plan

- Work top to bottom. Phases are ordered by dependency.
- Check a box `[x]` only when its **Verify** step passes.
- Update the **Resume here** marker and **Progress log** at the end of every session.
- Each intrinsic touches the same five layers; the per-intrinsic checklist in [Appendix A](#appendix-a-per-intrinsic-checklist) is the canonical "definition of done".
- Standard verification commands (run from repo root):
  - `cargo fmt`
  - `cargo build`
  - `cargo test --workspace`
  - `cargo test -p fpas-cli examples_pascal_test_suite_passes`

## Resume here

> **Next task:** Phase 4, Task **4.1** (`QueryRootViews`).
> **Last updated:** 2026-06-14
> **Notes:** Phase 3 done. Screen queries **367..=369** read the CRT back buffer via `Console::query_screen_line` / `query_screen_cell`. `ScreenCell` registered in sema; smoke test `tui_screen_query_test.fpas`.

---

## Reference: where each layer lives

| Layer | Path | Role |
| ----- | ---- | ---- |
| Intrinsic enum | `crates/fpas-bytecode/src/intrinsic/tui.rs` | Discriminants (`TuiIntrinsic`); next free range after `347` |
| Intrinsic enum test | `crates/fpas-bytecode/src/intrinsic/tests.rs` | Round-trip / coverage list |
| Sema registration | `crates/fpas-sema/src/std_registry/loaded/tui/` | `mod.rs`, `host_api.rs`, `application_api.rs`, `handlers.rs` |
| Compiler lowering | `crates/fpas-compiler/src/compiler/std_calls/tui/` | `mod.rs`, `views.rs`, `application.rs`, `host/` |
| VM execution | `crates/fpas-vm/src/vm/execute/io/tui/` | `views.rs`, `records.rs`, `host/`, `menu_bar_model.rs` |
| Shared state | `crates/fpas-vm/src/vm/shared.rs` | `TuiState`, `ViewRegistry` access |
| Std widgets / session | `crates/fpas-std/src/tui/` | `widget/menu_bar/`, `view/`, `session.rs` |
| Screen back buffer | `crates/fpas-std/src/console/` | `ConsoleState`, `screen_snapshot`, `test_cell` |
| VM-level tests | `crates/fpas-vm/src/tests/core/tui_host_vm/`, `tui_focus_vm/` | Bytecode-level host tests |
| Compiler-level tests | `crates/fpas-compiler/src/tests/std_library/tui*.rs` | Lowering + run tests |
| FPAS example tests | `examples/pascal/test/` | `*_test.fpas` + sidecars |
| User spec | `docs/pascal/std/tui-app.md` | Authoritative API doc |

---

## Phase 0 — Decisions and scaffolding

Resolve the [open decisions](README.md#open-decisions) before writing intrinsics, because they change signatures.

- [x] **0.1 Decide naming scheme.** Pick one prefix family (`Test*` for injection/pump, `Query*` for read, keep `Host*` only for mutators). Record the decision in `docs/pascal/std/tui-app.md`. **Verify:** decision written in spec; this plan's task names updated if they diverge.
- [x] **0.2 Decide `ViewId` representation.** Real opaque FPAS type vs bare `integer`. **Verify:** decision recorded in `README.md` open-decisions section with rationale.
- [x] **0.3 Decide `ScreenCell` color type.** Reuse `Std.Console` CRT color enum (`0..=15`) vs richer type. **Verify:** chosen type referenced in spec.
- [x] **0.4 Decide fate of sidecars.** Keep `*.script.toml` / `*.expect.screen` as sugar or deprecate. **Verify:** decision recorded; if deprecating, list affected files (`crates/fpas-cli/src/test_script/`, `crates/fpas-cli/src/cli_test/expect_screen.rs`).
- [x] **0.5 Reserve intrinsic discriminant range.** Allocate **356..=378** in `crates/fpas-bytecode/src/intrinsic/tui.rs` (skips **348..=355**, owned by `Std.Test`). **Verify:** `cargo build` passes; range comment present.

---

## Phase 1 — Headless deterministic host loop

Goal: open a TUI bound to a virtual screen and pump events one at a time, no terminal, no blocking `Run`.

- [x] **1.1 `Application.OpenForTest(Width, Height): Application`.** Open a `TuiSession` with a fixed-size virtual `ConsoleState` and no terminal writer. Reuse `Console::new` sizing path in `crates/fpas-std/src/console/mod.rs`. Follow [Appendix A](#appendix-a-per-intrinsic-checklist). **Verify:** VM test opens, asserts `QueryScreenSize` once 3.x exists; until then assert no panic + session active.
- [x] **1.2 `Application.TestPump(App)`.** Process exactly one queued event and settle the resulting redraw. Reuse `TuiHostProcessNext` + `TuiHostDispatchRedraw` in `crates/fpas-vm/src/vm/execute/io/tui_run.rs` and `tui/host/process.rs`. Guarantee back buffer reflects the event before returning. **Verify:** VM test: push a resize, pump, assert dispatched (via existing focus/resize observation).
- [x] **1.3 `Application.TestPumpUntilIdle(App)`.** Loop `TestPump` until the event queue is empty and no redraw is pending. **Verify:** VM test: push N events, one call drains all.
- [x] **1.4 `Application.CloseForTest(App)`.** Deterministic teardown mirroring `close_tui_application_state` (`crates/fpas-vm/src/vm/execute/io/tui/host/lifecycle.rs`). **Verify:** state cleared; re-open works in same program.
- [x] **1.5 Phase-1 FPAS smoke test.** `examples/pascal/test/tui_pump_test.fpas`: open → pump with no events → close, assert no error. **Verify:** `fpas test examples/pascal/test/tui_pump_test.fpas` passes.

---

## Phase 2 — Input injection from FPAS

Goal: inject keyboard/mouse/resize/paste/focus from FPAS, reusing `Vm::push_console_event`.

- [x] **2.1 `Application.TestSendKey(App, Key: KeyEvent)`.** Map to `ConsoleEvent::key`; reuse mapping in `crates/fpas-cli/src/test_script/console.rs`. **Verify:** VM test: send Escape, pump, `OnKeyPressed` observed.
- [x] **2.2 `Application.TestSendMouse(App, Event)`.** Full mouse event injection. **Verify:** VM test: send Down on a menu item, pump, `OnCommand` observed (mirror `std_tui_menu_bar_mouse_click_dispatches_on_command_over_desktop`).
- [x] **2.3 `Application.TestMoveMouse(App, X, Y)`.** Convenience for a `Move` action. **Verify:** VM test: move over bar item, pump, hover state changes (needs Phase 4/5 to assert; until then assert dispatch tag).
- [x] **2.4 `Application.TestClickMouse(App, X, Y)`.** `Down` then `Up` at one point. **Verify:** VM test: click bar item with submenu opens it.
- [x] **2.5 `Application.TestResize / TestPaste / TestFocus`.** One intrinsic each, reusing existing `ConsoleEvent` constructors. **Verify:** VM test per event type: handler observed after pump.
- [x] **2.6 Phase-2 FPAS test.** `examples/pascal/test/tui_inject_key_test.fpas`: open → `TestSendKey(Escape)` → pump → assert `OnKeyPressed` flag. **Verify:** `fpas test` passes.

---

## Phase 3 — Screen and cell introspection

Goal: read the CRT back buffer (chars + colors) as FPAS values.

- [x] **3.1 Define `ScreenCell` and `Size` (if missing) in `Std.Tui` registry.** Add record types in `crates/fpas-sema/src/std_registry/loaded/tui/`. **Verify:** `cargo build`; type usable in a test program.
- [x] **3.2 `Application.QueryScreenSize(App): Size`.** From `ConsoleState` width/height. **Verify:** open 80x25, assert returned size.
- [x] **3.3 `Application.QueryScreenLine(App, Y): string`.** Row characters; reuse `ScreenSnapshot` row access in `crates/fpas-std/src/console/snapshot.rs`. **Verify:** paint known text, assert line.
- [x] **3.4 `Application.QueryScreenCell(App, X, Y): ScreenCell`.** Char + fg + bg; reuse `ConsoleState` cell access used by `console.test_cell`. **Verify:** paint a menu bar, assert accel-letter cell color matches `menu_bar_paints_shortcut_letter_in_accel_color`.
- [x] **3.5 Phase-3 FPAS test.** `examples/pascal/test/tui_screen_query_test.fpas`: paint text via `OnPaint`, pump, assert line + a colored cell. **Verify:** `fpas test` passes.

---

## Phase 4 — View tree and widget-state introspection

Goal: expose `ViewRegistry` and `MenuBarWidget` internals as FPAS values.

- [ ] **4.1 `Application.QueryRootViews(App): array of ViewId`.** From `ViewRegistry` roots (`crates/fpas-std/src/tui/view/`). **Verify:** create two root views, assert count/order.
- [ ] **4.2 `Application.QueryViewRect(App, ViewId): Rect`.** Absolute rect; reuse `ViewRegistry::rect`. **Verify:** create view at known coords, assert rect.
- [ ] **4.3 `Application.QueryViewParent(App, ViewId): Option of ViewId`.** **Verify:** reparent, assert parent.
- [ ] **4.4 `Application.QueryViewChildren(App, ViewId): array of ViewId`.** **Verify:** push children, assert list + z-order.
- [ ] **4.5 Define `MenuBarState` record in registry.** Fields per design (`menuActive`, `hoveredIndex`, `submenuOpen`, `submenuBarIndex`, `selectedEntry`). **Verify:** `cargo build`.
- [ ] **4.6 Expose `MenuBarWidget` state.** Add getters in `crates/fpas-std/src/tui/widget/menu_bar/` (e.g. `mod.rs`) for hovered/open/selected; keep fields private, expose a snapshot struct. **Verify:** Rust unit test reads snapshot.
- [ ] **4.7 `Application.QueryMenuBarState(App, ViewId): MenuBarState`.** Map widget snapshot → FPAS record. **Verify:** VM test: open submenu via key, assert `submenuOpen = true`.
- [ ] **4.8 Phase-4 FPAS test.** `examples/pascal/test/tui_view_query_test.fpas`: create menu bar, query rect + initial menu state. **Verify:** `fpas test` passes.

---

## Phase 5 — Host behavior fixes (prerequisites for hover tests)

These are real host changes, not test plumbing. Without them, hover has nothing to assert.

- [ ] **5.1 Submenu mouse hover.** In `crates/fpas-std/src/tui/widget/menu_bar/input.rs`, `handle_mouse`: when a popup is open and action is `Move` over a popup row, update the selected entry and return `HoverChanged` (mirror keyboard `Up`/`Down`). Currently non-`Down` events return `Ignored`. **Verify:** Rust unit test `menu_bar_submenu_mouse_move_changes_selection` in `widget/menu_bar/tests.rs`.
- [ ] **5.2 Bar item hover via `Move`.** Confirm/repair `Move` over a bar item sets `hovered` and paint reflects highlight colors. **Verify:** Rust unit test asserting hovered index + a `test_cell` color after a `Move`.
- [ ] **5.3 Redraw determinism after pump.** Ensure `TestPump` flushes the back buffer so the next FPAS query sees post-event state (ties to 1.2). **Verify:** FPAS test: move → pump → `QueryScreenCell` shows highlight in the same program.

---

## Phase 6 — Std.Test sugar (optional but recommended)

Pure wrappers over Phase 3/4 queries. Live in `crates/fpas-std/src/test/` + sema/compiler/bytecode wiring like other `Std.Test` procedures.

- [ ] **6.1 `AssertScreenLine(Expected: string; Y: integer)`.** **Verify:** passing and failing cases (failing emits `F4023`).
- [ ] **6.2 `AssertScreenCell(X, Y: integer; Ch: char; Fg, Bg: Color)`.** **Verify:** pass/fail cases.
- [ ] **6.3 `AssertViewRect(App, V: ViewId; X, Y, W, H)`.** **Verify:** pass/fail cases.
- [ ] **6.4 Update `docs/pascal/std/test.md`** with the new assertions. **Verify:** spec lists them; links valid.

---

## Phase 7 — Capstone: full menu hover test in FPAS

The target experience from the design doc, fully under `fpas test`.

- [ ] **7.1 `examples/pascal/test/tui_menu_hover_test.fpas`.** Reproduce the README target program: open headless, move over "File", pump, assert highlight cell + `hoveredIndex`; click to open submenu, move over entry, pump, assert `submenuOpen` + `selectedEntry`. **Verify:** `fpas test examples/pascal/test/tui_menu_hover_test.fpas` passes.
- [ ] **7.2 Add to test discovery.** Ensure it runs under `examples_pascal_test_suite_passes`. **Verify:** `cargo test -p fpas-cli examples_pascal_test_suite_passes`.
- [ ] **7.3 Port one real menu regression.** Convert the bug that motivated this work into a native FPAS test. **Verify:** test reproduces the bug pre-fix and passes post-fix.

---

## Phase 8 — Cleanup, docs, and removals

- [ ] **8.1 Apply Phase-0 deprecations.** If sidecars are being dropped, remove/retire the relevant code in `crates/fpas-cli/src/test_script/` and `cli_test/expect_screen.rs`, and migrate existing TUI tests. **Verify:** `cargo test --workspace` green; no orphaned modules.
- [ ] **8.2 Update `docs/pascal/std/tui-app.md`.** Document every new intrinsic, record type, and the headless test flow. Add bytecode discriminants list entry. **Verify:** `tui_rust_sources_link_to_pascal_spec_docs` test passes; `cargo test -p fpas-vm tui_spec_links`.
- [ ] **8.3 Update `docs/future/README.md` status.** Mark this proposal as implemented or in-progress. **Verify:** link + status accurate.
- [ ] **8.4 Update this plan's status header** to `complete` and finalize the progress log.

---

## Appendix A — per-intrinsic checklist

For **every** new intrinsic, all of these must be done before its task box is checked:

- [ ] Discriminant added in `crates/fpas-bytecode/src/intrinsic/tui.rs` with a doc comment (stack effect + spec link).
- [ ] Listed in `crates/fpas-bytecode/src/intrinsic/tests.rs` coverage.
- [ ] Sema signature registered in `crates/fpas-sema/src/std_registry/loaded/tui/` (correct param/return types, LLM-friendly diagnostics).
- [ ] Lowering in `crates/fpas-compiler/src/compiler/std_calls/tui/` emits the intrinsic.
- [ ] Execution implemented in `crates/fpas-vm/src/vm/execute/io/tui/` (with `with_tui` / lock-ordering rules from `shared.rs`).
- [ ] Symbol name added in `crates/fpas-std/src/std_units/symbols/std_symbols.rs`.
- [ ] VM-level test in `crates/fpas-vm/src/tests/core/tui_host_vm/` (or `tui_focus_vm/`).
- [ ] Compiler-level lowering/run test in `crates/fpas-compiler/src/tests/std_library/tui*.rs`.
- [ ] Documented in `docs/pascal/std/tui-app.md`.
- [ ] `cargo fmt && cargo build && cargo test --workspace` green.

## Appendix B — intrinsic inventory

Running list of new intrinsics and their assigned discriminants (reserved in Phase 0.5).

| Intrinsic | Discriminant | Phase | Done |
| --------- | ------------ | ----- | ---- |
| `OpenForTest` | 356 | 1.1 | [x] |
| `TestPump` | 357 | 1.2 | [x] |
| `TestPumpUntilIdle` | 358 | 1.3 | [x] |
| `CloseForTest` | 359 | 1.4 | [x] |
| `TestSendKey` | 360 | 2.1 | [x] |
| `TestSendMouse` | 361 | 2.2 | [x] |
| `TestMoveMouse` | 362 | 2.3 | [x] |
| `TestClickMouse` | 363 | 2.4 | [x] |
| `TestResize` | 364 | 2.5 | [x] |
| `TestPaste` | 365 | 2.5 | [x] |
| `TestFocus` | 366 | 2.5 | [x] |
| `QueryScreenSize` | 367 | 3.2 | [x] |
| `QueryScreenLine` | 368 | 3.3 | [x] |
| `QueryScreenCell` | 369 | 3.4 | [x] |
| `QueryRootViews` | 370 | 4.1 | [ ] |
| `QueryViewRect` | 371 | 4.2 | [ ] |
| `QueryViewParent` | 372 | 4.3 | [ ] |
| `QueryViewChildren` | 373 | 4.4 | [ ] |
| `QueryMenuBarState` | 374 | 4.7 | [ ] |
| *(spare)* | 375..=378 | — | — |

**348..=355** are reserved for `Std.Test` (`TestIntrinsic`); do not assign TUI testing discriminants in that range.

Renames (no new discriminant): `QueryFocusedViewId` replaces Pascal name for **282**; `QueryModalDepth` replaces Pascal name for **278**.

## Progress log

Append one entry per working session: date, tasks completed, surprises, and the next task to resume from.

- **2026-06-14:** Completed **Phase 3** (3.1–3.5). Registered `ScreenCell`; added `QueryScreenSize` / `QueryScreenLine` / `QueryScreenCell` (**367..=369**); `Console::query_screen_line` / `query_screen_cell`; VM + compiler tests; `examples/pascal/test/tui_screen_query_test.fpas`. Next: **4.1** `QueryRootViews`.
- **2026-06-14:** Completed **Phase 2** (2.1–2.6). Added input injectors `TestSendKey` … `TestFocus` (**360..=366**); `pop_console_event` in VM; `examples/pascal/test/tui_inject_key_test.fpas`. Next: **3.1** `ScreenCell` registry.
- **2026-06-14:** Completed **Phase 1** (1.1–1.5). Added `OpenForTest`, `TestPump`, `TestPumpUntilIdle`, `CloseForTest` (discriminants **356..=359**); headless `TuiSession::open_for_test`; `examples/pascal/test/tui_pump_test.fpas`. Corrected intrinsic range to **356..=378** (collision with `Std.Test` at **348..=355**). Next: **2.1** `TestSendKey`.
- **2026-06-14:** Completed **0.3–0.5** (Phase 0 done). `ScreenCell` uses CRT `0..=15` + `Std.Console` constants; TUI sidecars deprecated (remove Phase 8); reserved intrinsics **356..=378** in `tui.rs`. Next: **1.1** `OpenForTest`.
- **2026-06-14:** Completed **0.2** — `ViewId` decided as real opaque FPAS type (`Std.Tui.ViewId`, empty record like `Application`). `Option of ViewId` replaces integer `-1` for focus/parent detach. Documented in `tui-app.md` § ViewId type. Next: **0.3** (`ScreenCell` color type).
- **2026-06-14:** Completed **0.1** — naming convention decided and documented in `docs/pascal/std/tui-app.md` § Native TUI testing API. Scheme: `Test*` (pump/inject/lifecycle), `Query*` (read-only), `Host*` (mutators). Planned renames: `HostQueryFocusedViewId` → `QueryFocusedViewId`, `HostModalDepth` → `QueryModalDepth`. Next: **0.2** (`ViewId` type decision).
