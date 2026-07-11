# Remaining work (Phase 7 closure)

Ordered backlog after the 2026-07-11 symbol audit. Public API and docs live under `docs/pascal/std/tui/`; this file is the implementation handoff only.

**Branch:** `refactor/tui-try-2`  
**Last synced:** 2026-07-11

## Summary

| Stream | Status | Blocks Phase 7 sign-off? |
| --- | --- | --- |
| A — Upstream read-back adapters | **Blocked** on `turbo-vision` `v2.0.0` | Yes |
| B — Interim test API cleanup | **Done (2026-07-11)** | No |
| C — Test file rename (`*_try2_*`) | **Done (2026-07-11)** | No |
| D — Plan archive | After A + verification green | Yes |

Root bridge migration is **complete**: `crates/fpas-vm/src/vm/execute/io/tui/` contains only `mod.rs` and `try2/`.

---

## Stream A — Remove three `bridged_*` adapters (upstream-dependent)

**Files (intentional exceptions today):**

- `try2/bridged_check_box.rs`
- `try2/bridged_radio_button.rs`
- `try2/bridged_outline.rs`

**Why they remain:** At pin `v2.0.0`, `CheckBox`, `RadioButton`, and `OutlineViewer` do not expose a supported live-state read-back hook through `dyn View` (no reliable `as_any_mut` downcast). The adapters copy keyboard/mouse selection into FPAS host cells so `CheckBox.Checked`, `RadioButton.Selected`, `Outline.Selection`, and `Outline.SelectedText` stay correct after interactive input.

**Done when:**

1. Upstream adds downcast or a documented read-back API for those three types **or** FPAS bumps to a revision that includes it.
2. Replace adapter construction in `try2/views/{check_box,radio_button,outline}.rs` with direct upstream views (same pattern as `ListBox`, `Button`, `StaticText`).
3. Delete the three `bridged_*.rs` files.
4. Re-run regressions: `tests/tui/views/check_box_test.fpas`, `radio_button_test.fpas`, `outline_*_test.fpas`, `tests/tui/events/check_box_mouse_test.fpas`, `radio_button_mouse_test.fpas`.

**Optional upstream issue text:** expose `View::as_any_mut` (or equivalent) on checkbox, radio, and outline types so embedders can sync external state after `handle_event`.

---

## Stream B — Finalize headless test API (no upstream dependency)

**Registered Pascal surface:**

| Symbol | Role |
| --- | --- |
| `Test.Click` | Mouse click at button center |
| `Test.DispatchMenu` | Menu bar item → command id |
| `Test.InjectCommand` | Queue command for next headless `Run` turn |
| `Test.InjectKeyboard` | Queue Turbo Vision key code |
| `Application.TestClickMouse` | Screen-coordinate click (+ stateful control toggle) |
| `Application.TestSetDialogResult` | Stub queue for headless `MessageBox` |
| `Application.TestSetFileDialogResult` | Stub queue for headless `RunFileDialog` |

Bytecode intrinsics keep historical Rust names (`TestClickButton`, `Try2InjectCommand`, …) to avoid opcode renumbering.

**Implementation order:**

1. [x] Add `Std.Tui.Test.Click` in sema + symbols (alias to `TestClickButton` intrinsic).
2. [x] Migrate `tests/tui/` and `apps/ide/tests/` to `Test.*` helpers.
3. [x] Add `Test.DispatchMenu`, `Test.InjectCommand`, and `Test.InjectKeyboard`.
4. [x] Update [docs/pascal/std/tui/app/testing.md](../pascal/std/tui/app/testing.md) to show target API only.
5. [x] Remove interim `Application.TestClickButton`, `TestDispatchMenuCommand`, `TestInjectCommand`, and `TestInjectKeyboard` from symbol table and sema (2026-07-11).

Optional later: unified `Test.InjectEvent` when `Std.Console` event records are wired.

---

## Stream C — Drop `_try2` test suffix

All 37 try-1 `tests/tui/controls/*` files are gone. The 30 Try-2 regression files now use the standard `*_test.fpas` naming (2026-07-11).

**Steps:**

1. [x] Rename `tests/tui/**/*_try2_test.fpas` → `*_test.fpas` (preserve theme subdirs).
2. [x] [`tests/suite.fpasprj`](../../tests/suite.fpasprj) already globbed `tui/**/*_test.fpas` — no change required.
3. [x] Run `fpas test tests/tui/` and `cargo test -p fpas-cli fpas_regression_suite_passes`.

Safe to do in the same PR as Stream B or immediately after.

---

## Stream D — Close the rewrite plan

**Prerequisites:**

- Stream A complete (`rg bridged_ crates/` → no matches).
- [verification.md](verification.md) checklist green (grep invariants, docs, tests, manual smoke).
- [migration-phases.md](migration-phases.md) Phase 7 boxes checked.

**Then:**

1. Add completion banner to [README.md](README.md): `Status: completed — see docs/pascal/std/tui/`.
2. Move historical baseline/problem docs to `docs/future/` **or** keep this directory as archive with a one-line pointer at the top.
3. Remove stale coexistence / try-1 references from [README.md](README.md) quick comparison table.

---

## Secondary blocker (non-blocking for A/B/C)

**Headless `RunFileDialog`:** Live path uses upstream `FileDialog::execute`. Headless tests use `Try2Session` queue via `TestSetFileDialogResult` because constructing a full upstream `Application` over the headless terminal is not available at the current pin. Documented in [status.md](status.md#blockers). Replace the queue when upstream or VM exposes a headless-safe execute path.

---

## Suggested next session (agent pick)

1. ~~**Stream B step 1** — register `Std.Tui.Test.Click`~~ **Done (2026-07-11).**
2. ~~**Stream B step 2** — migrate FPAS tests from `Application.TestClickButton` to `Test.Click`.~~ **Done (2026-07-11).**
3. ~~**Stream B step 3** — add `Test.DispatchMenu`, `Test.InjectCommand`, and `Test.InjectKeyboard` aliases; migrate call sites.~~ **Done (2026-07-11).** Remaining: unified `Test.InjectEvent` (optional); remove interim `Application.Test*` names (step 5).
4. ~~Leave Stream A parked until upstream contact or bump.~~
5. ~~**Stream C** — rename 30 `*_try2_test.fpas` files to `*_test.fpas`.~~ **Done (2026-07-11).**

---

## Verification commands (quick)

```bash
cargo test -p fpas-vm try2::
cargo test -p fpas-sema std_units::tui
cargo test -p fpas-cli fpas_regression_suite_passes
fpas test tests/tui/
fpas test apps/ide/tests/
rg "TurboVisionObject|pending_reconcile|FPAS_TV_COMMAND_OFFSET|TuiCreateDialog" crates/
rg "bridged_" crates/    # expect exactly three files until Stream A
```
