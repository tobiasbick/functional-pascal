# Migration phases

Ordered implementation plan. Each phase has **exit criteria** — do not start the next phase until they pass.

Estimates assume focused hobby-project pace (part-time).

---

## Phase 0 — Prep (0.5–1 day)

**Work**

- [x] Read this plan directory end-to-end.
- [x] Inventory `test_util` exports at `v2.0.0` — see [baseline.md](baseline.md#upstream-test-util-v200).
- [x] Freeze try-1: rewrite work on `refactor/tui-try-2` only; `main` unchanged after plan merge.
- [x] Create working branch `refactor/tui-try-2`.

**Exit criteria**

- Branch exists; team agrees on [target-api.md](target-api.md) naming (`New`, `ExecView`, `CM_*`).

---

## Phase 1 — Foundation (2–3 days) — **complete** (2026-07-07)

**Work**

- [x] Add `ViewRegistry` in `try2/registry.rs` (wired on `Worker` via `Try2Session`).
- [x] ~~Slim `TuiState` on Worker~~ — **deferred to phase 7** ([rust-layout.md](rust-layout.md)): requires deleting try-1 `TurboVisionState`; `Try2Session` on `Worker` is the coexistence bridge until then.
- [x] Implement `try2/session.rs` integration with `TuiSession.open` / `OpenForTest`
- [x] Implement `try2/geometry.rs` rect conversion.
- [x] Add `fpas-std/tui/cm_constants.rs` with core `CM_*` constants.
- [x] Sema + compiler for `Application.New` / `Close` — `Application.New` → `ApplicationOpen`; `Close` / `CloseForTest` reuse try-1 intrinsics + `try2.reset()` on close.
- [x] Rust unit tests: registry allocate/validate/clear.

**Exit criteria**

- [x] `cargo build` passes.
- [x] `cargo test -p fpas-vm` registry tests pass.
- [x] No *public* `docs/pascal/` spec for try-2 yet (internal/refactor docs only; try-2 Pascal symbols coexist on branch).

---

## Phase 2 — Vertical slice: Button + Dialog + ExecView (3–4 days) — **complete** (2026-07-07)

**Work**

- [x] `Dialog.NewModal`, `Dialog.Add(Button)` — Rust internal (`try2/views/`)
- [x] `Button.New`, `Dialog.Add` — Pascal API + intrinsics (477/478); smoke test uses target pattern
- [x] `Application.ExecView` → upstream `exec_view` (headless via `HeadlessTvApp::exec_modal_view`; interactive via `try2/app.rs`)
- [x] `headless.rs`: headless modal loop + CRT export (`try2/headless.rs`, `headless_tv_draw.rs`)
- [x] `Test.InjectEvent` or click helper for button command (`Application.TestClickButton` try-2 path).
- [x] One FPAS test: modal OK returns `CM_OK` (`tests/tui/smoke/modal_button_try2_test.fpas`).
- [x] `events.rs`: `OnCommand` dispatch without offset translation (`try2/events.rs`)
- [x] `Application.Run` on try-2 path (`try2/run.rs`; uses `Application.OnCommand` until 2-arg `Run` lands)

**Exit criteria**

- [x] `fpas test tests/tui/smoke/modal_button_try2_test.fpas` passes.
- [x] Interactive smoke: manual run of tiny program in terminal — `fpas run examples/pascal/tui/modal_button_try2.fpas` (OK/Cancel/× with mouse or Enter/Esc).

---

## Phase 3 — Run loop + Desktop + Window (2–3 days) — **complete** (2026-07-08)

**Work**

- [x] `Application.Run` with `OnCommand` callback parameter — `Application.Run(App, OnCommand)` sema + intrinsic 484.
- [x] `Application.Quit` → stop run loop — **partial:** `quit_requested` honored on try-2 path; live `app.running` wiring TBD.
- [x] `Window.New`, `Window.Add`, `Desktop.Add` — intrinsics 480–482; `tests/tui/smoke/window_quit_try2_test.fpas`.
- [x] `StaticText`, menu/status chrome (`chrome.rs`) — `StaticText.New`, `MenuBar.New`, `StatusLine.New`, `SetMenuBar`/`SetStatusLine` routing.
- [x] FPAS tests: modeless window + quit command — `window_quit_try2_test.fpas` (button click → `CM_QUIT` → `Application.Quit`).

**Exit criteria**

- [x] Port simplified `examples/pascal/tui/turbo_vision_window.fpas` to new API — `examples/pascal/tui/turbo_vision_window_try2.fpas`.
- [x] `cargo test --workspace` passes (try-1 `tests/tui/controls/*` must stay green until phase 7 — coexistence routing).

---

## Phase 4 — Remaining phase-1 widgets (3–4 days) — **complete**

**Work**

- [x] `InputLine`, `CheckBox`, `ListBox`, `RadioButton`, `Memo`, `TextViewer` — `*.New`, `Dialog.Add` / `Window.Add`, read-back + setters.
- [x] Read-back methods: `InputLine.Text`, `CheckBox.Checked`, `ListBox.Selection`, `RadioButton.Selected` (where applicable).
- [x] Runtime setters: `InputLine.SetText`, `CheckBox.SetChecked`, `ListBox.SetItems`, `RadioButton.SetSelected`, `Memo.SetText`, `TextViewer.SetText`.
- [x] FPAS tests: `tests/tui/views/*_try2_test.fpas` for all phase-1 widgets.

**Exit criteria**

- New test suite covers all phase-1 widgets in [upstream-mapping.md](upstream-mapping.md).
- Delete corresponding old `tests/tui/controls/tui_turbo_vision_*` files as each is replaced (pending cleanup pass).

---

## Phase 5 — Modals and helpers (2 days) — **complete for current branch scope**

**Work**

- [x] `Application.MessageBox` (try-2 headless via `try2_headless_exec_view`; live via upstream `message_box`)
- [x] `Application.RunFileDialog` (live path done; headless uses a Try-2-local queued adapter until upstream exposes a headless-safe `FileDialog::execute` host)
- [x] `OnKey` / `OnMouse` optional hooks (`OnKey` headless + live; `OnMouse` live; export `Application.OnKey` / `OnMouse` from `Std.Tui`)

**Exit criteria**

- [x] Port `examples/pascal/tui/message_box.fpas` and add a try-2 file dialog example (`examples/pascal/tui/file_dialog_try2.fpas`).
- [x] No new Try-2 modal tests depend on the try-1 dialog queues (`tests/tui/modals/message_box_try2_test.fpas` uses `Try2InjectKeyboard`; Try-2 `RunFileDialog` consumes `Try2Session` state, not try-1 `test_file_dialog_result`).
- [ ] Rename or replace interim `TestSetDialogResult` / `TestSetFileDialogResult` helpers with the final `Test.*` event API during Phase 7/8 public testing cleanup.

---

## Phase 6 — IDE migration (2–4 days) — **complete**

**Work**

- [x] Rewrite `apps/ide/src/` per [ide-migration.md](ide-migration.md) for menu, status, About, Open, and Exit.
- [x] Rewrite `apps/ide/tests/` for try-2 run helpers and command injection.
- [x] Automated terminal checklist coverage for TUI sema/compiler/doc links, try-1 controls coexistence, IDE headless flows, and relevant FPAS formatting.
- [x] Manual terminal checklist in a real terminal for IDE menus, About, Open file, Exit, and resize (signed off 2026-07-09).

**Exit criteria**

- [x] `fpas test apps/ide/tests/` all pass.
- [x] Automated checklist evidence recorded in [status.md](status.md#verified-2026-07-09).
- [x] IDE usable interactively (manual sign-off 2026-07-09).

---

## Phase 7 — Delete try-1 (1–2 days) — **in progress**

**Work**

- [ ] Remove all modules in [deletion-checklist.md](deletion-checklist.md).
- [ ] **Slim `TuiState`**: drop `TurboVisionState`, `TurboVisionObject`, snapshot structs ([rust-layout.md](rust-layout.md)).
- [ ] Remove old bytecode intrinsics and sema symbols.
- [ ] Remove old `docs/pascal/std/tui/` pages; write new spec from [target-api.md](target-api.md).
- [ ] Update skill, AGENTS.md bridge pointers, `.cursor/rules` TUI examples.
- [ ] Delete or archive `docs/refactor-tui-try-2/` (or mark completed in README).
- [x] Phase-1 widget + run/chrome/modal try-1 control tests removed (**20/37**); replacements in `tests/tui/views/`, `tests/tui/smoke/`, `tests/tui/modals/`. **17** controls tests remain.

**Exit criteria**

- [verification.md](verification.md) checklist all green.
- `rg TurboVisionObject` / `rg pending_reconcile` / `rg bridged_` returns no matches in `crates/`.

---

## Phase 8 — Optional follow-ups

Not blocking completion:

- [ ] `Application.Configure` + `ApplicationHandlers` (Graph parity).
- [ ] `Outline` and remaining phase-2 widgets.
- [ ] `EditorWindow` for IDE editor pane.
- [ ] Generate `CM_*` from upstream build script to avoid drift.

---

## Risk mitigations per phase

| Risk | Mitigation |
| --- | --- |
| `SetText` on live view without bridged wrapper | Test early in phase 4; read upstream mutators on each type |
| Headless modal loops hang | Use `put_event` + timeout in tests |
| Radio button grouping | Port `GroupId` cell logic minimally or bind `Cluster` |
| Large bang rewrite | Phase 2 vertical slice proves architecture before widget flood |

---

## Parallel work rules

- **Do not** maintain two public APIs. Internal-only stubs until phase 2 exit, then switch tests wholesale.
- **Do** keep `cargo build` green after each commit where possible; use `#[cfg]` only briefly if needed.
- Update this file’s checkboxes as phases complete.
