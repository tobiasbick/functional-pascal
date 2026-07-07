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

## Phase 1 — Foundation (2–3 days)

**Work**

- [x] Add `ViewRegistry` in `try2/registry.rs` (coexists with try-1; not wired to Worker yet).
- [ ] Slim `TuiState` on Worker ([rust-layout.md](rust-layout.md)) — `Try2Session` on Worker landed; full `TuiState` slimming deferred
- [x] Implement `try2/session.rs` integration with `TuiSession.open` / `OpenForTest`
- [x] Implement `try2/geometry.rs` rect conversion.
- [x] Add `fpas-std/tui/cm_constants.rs` with core `CM_*` constants.
- [ ] Sema + compiler stubs for `Application.New` / `Close` only.
- [x] Rust unit tests: registry allocate/validate/clear.

**Exit criteria**

- `cargo build` passes.
- `cargo test -p fpas-vm` registry tests pass.
- No user-facing API published yet (internal only).

---

## Phase 2 — Vertical slice: Button + Dialog + ExecView (3–4 days)

**Work**

- [x] `Dialog.NewModal`, `Dialog.Add(Button)` — Rust internal (`try2/views/`)
- [x] `Application.ExecView` → upstream `exec_view` (headless via `HeadlessTvApp::exec_modal_view`; interactive via `try2/app.rs`)
- [x] `headless.rs`: headless modal loop + CRT export (`try2/headless.rs`, `headless_tv_draw.rs`)
- [ ] `Test.InjectEvent` or click helper for button command.
- [ ] One FPAS test: modal OK returns `CM_OK`.
- [ ] `events.rs`: `OnCommand` dispatch without offset translation.

**Exit criteria**

- `fpas test tests/tui/smoke/modal_button_test.fpas` (new minimal test) passes.
- Interactive smoke: manual run of tiny program in terminal (document in phase notes).

---

## Phase 3 — Run loop + Desktop + Window (2–3 days)

**Work**

- [ ] `Application.Run` with `OnCommand` callback parameter.
- [ ] `Application.Quit` → `app.running = false`.
- [ ] `Window.New`, `Window.Add`, `Desktop.Add`.
- [ ] `StaticText`, menu/status chrome (`chrome.rs`).
- [ ] FPAS tests: modeless window + quit command.

**Exit criteria**

- Port simplified `examples/pascal/tui/turbo_vision_window.fpas` to new API.
- `cargo test --workspace` passes (old TUI tests may still fail — expected).

---

## Phase 4 — Remaining phase-1 widgets (3–4 days)

**Work**

- [ ] `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer`.
- [ ] Read-back methods: `InputLine.Text`, `ListBox.Selection`, etc.
- [ ] Runtime setters: `SetText`, `SetItems`, `SetChecked`, `SetTitle`.
- [ ] FPAS tests: one file per widget under `tests/tui/views/` (new layout).

**Exit criteria**

- New test suite covers all phase-1 widgets in [upstream-mapping.md](upstream-mapping.md).
- Delete corresponding old `tests/tui/controls/tui_turbo_vision_*` files as each is replaced.

---

## Phase 5 — Modals and helpers (2 days)

**Work**

- [ ] `Application.MessageBox`
- [ ] `Application.RunFileDialog`
- [ ] `OnKey` / `OnMouse` optional hooks

**Exit criteria**

- Port `examples/pascal/tui/message_box.fpas` and `file_dialog` example.
- No `TestSetDialogResult` / `TestSetFileDialogResult` in new tests.

---

## Phase 6 — IDE migration (2–4 days)

**Work**

- [ ] Rewrite `apps/ide/src/` per [ide-migration.md](ide-migration.md).
- [ ] Rewrite `apps/ide/tests/`.
- [ ] Manual terminal checklist for IDE menus, About, Open file.

**Exit criteria**

- `fpas test apps/ide/tests/` all pass.
- IDE usable interactively (manual sign-off).

---

## Phase 7 — Delete try-1 (1–2 days)

**Work**

- [ ] Remove all modules in [deletion-checklist.md](deletion-checklist.md).
- [ ] Remove old bytecode intrinsics and sema symbols.
- [ ] Remove old `docs/pascal/std/tui/` pages; write new spec from [target-api.md](target-api.md).
- [ ] Update skill, AGENTS.md bridge pointers, `.cursor/rules` TUI examples.
- [ ] Delete or archive `docs/refactor-tui-try-2/` (or mark completed in README).

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
