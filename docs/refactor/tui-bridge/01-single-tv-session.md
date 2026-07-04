# 01 — Single Turbo Vision session per FPAS application

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** High — do this first among open TUI bridge items.

**Depends on:** [done/01-turbo-vision-2-upgrade.md](done/01-turbo-vision-2-upgrade.md) (completed)

**Blocks:** Correct modal stacking, shared menu/status during `ExecDialog`, cleaner `RunFileDialog`.

## Problem

Today FPAS keeps authoritative state in `TurboVisionState`, but **three separate** `turbo_vision::app::Application` instances can exist:

| Call site | File | Behavior |
| --- | --- | --- |
| `Application.Run` | `tv_run.rs` | Long-lived interactive loop |
| `Application.ExecDialog` | `exec_dialog.rs` | New `Application::new()` per modal |
| `Application.RunFileDialog` | `file_dialog.rs` | New `Application::new()` per picker |

Modals do not share the running session’s desktop, menu bar, or status line. Terminal init/shutdown may run more than once. Behavior diverges from native Turbo Vision programs.

## Target

One upstream `Application` (or equivalent session object) for the lifetime of FPAS `Application.Open` … `Close` / end of `Run`:

- `Run` drives the event loop on that instance.
- `ExecDialog` / `RunFileDialog` execute modals **on the same** instance (or on its desktop modal stack).
- Headless mode may still stub events, but should not spawn unrelated TV apps for modals when a session already exists.

## Tasks

- [ ] **Design** — Where to store the live `TurboVisionApplication` (e.g. `TuiSession`, `TurboVisionState`, or run-local `ApplicationInteractiveSession` extended to session scope). Document ownership and re-entrancy (`ExecDialog` from `OnCommand` during `Run`).
- [ ] **Run path** — Build/populate TV app once at Run start; reuse for the loop in `interactive_loop.rs`.
- [ ] **ExecDialog** — Remove standalone `Application::new()` in `exec_dialog.rs`; execute modal on session app. Preserve read-back (`InputText`, `Checked`, …) via existing cells/bindings.
- [ ] **RunFileDialog** — Same for `file_dialog.rs`.
- [ ] **Headless** — Define behavior when `OpenForTest` + `ExecDialog`: either reuse mock terminal on one session or keep explicit test stubs (`TestSetDialogResult`) without a second app.
- [ ] **Lifecycle** — Ensure terminal shutdown happens once on `Close` / end of `Run`; no leak on error paths.
- [ ] **Tests** — `tests/tui/controls/tui_turbo_vision_exec_dialog_test.fpas`, file dialog tests, IDE About menu test; add regression if modal during `Run` was broken before.
- [ ] **Docs** — Update [docs/pascal/std/tui/app/modals.md](../../pascal/std/tui/app/modals.md) / [lifecycle.md](../../pascal/std/tui/app/lifecycle.md) if observable rules change.
- [ ] **Context** — Update [00-context.md](00-context.md) “Known duplication” section when done.

## Files (expected touch)

```text
crates/fpas-vm/src/vm/execute/io/tui/
  tv_run.rs
  exec_dialog.rs
  file_dialog.rs
  interactive_loop.rs
  application.rs / lifecycle.rs (if session storage moves)
crates/fpas-vm/src/vm/shared/tui.rs
crates/fpas-std/src/tui/ … (session if needed)
```

## Verification

```text
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_exec_dialog_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_file_dialog_test.fpas
cargo run -q -p fpas-cli -- test apps/ide/tests/
```

Manual: run IDE, open Help → About during live session; menu/status should remain consistent.

## Notes

- Keep the FPAS handle graph as source of truth for Pascal `Create*`; this item changes **when** and **how** snapshots become TV views, not the public Pascal API.
- If upstream adds a documented “run modal on existing Application” helper in a future release, prefer it over custom wiring.
