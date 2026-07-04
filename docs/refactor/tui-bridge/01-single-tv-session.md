# 01 — Single Turbo Vision session per FPAS application

**Status:** [ ] Not started · [x] In progress · [ ] Done

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

- [x] **Design** — Live `TurboVisionApplication` on main [`Worker`](../../../crates/fpas-vm/src/vm/worker.rs) (`live_turbo_vision_app`, `!Send`). Helpers in `session_app.rs`. Re-entrancy: per-step borrows in `turbo_vision_drive_live_interactive_loop`; FPAS dispatch runs without holding `&mut Application`.
- [x] **Run path** — `tv_run.rs` calls `turbo_vision_refresh_live_desktop` + `turbo_vision_drive_live_interactive_loop` (no per-Run `Application::new()`).
- [x] **ExecDialog** — `exec_dialog.rs` uses `turbo_vision_with_live_app`; read-back unchanged.
- [x] **RunFileDialog** — `file_dialog.rs` uses `turbo_vision_with_live_app`.
- [x] **Headless** — Unchanged: `TestSetDialogResult` / `TestSetFileDialogResult` stubs; no live TV app in headless mode.
- [x] **Lifecycle** — `turbo_vision_shutdown_live_app` on `Application.Close` and `Application.Open` reset (`lifecycle.rs`).
- [x] **Tests** — `cargo test --workspace`; exec dialog, file dialog, IDE tests (6/6) pass.
- [ ] **Docs** — Update [docs/pascal/std/tui/app/modals.md](../../pascal/std/tui/app/modals.md) / [lifecycle.md](../../pascal/std/tui/app/lifecycle.md) if observable rules change.
- [ ] **Context** — Update [00-context.md](00-context.md) “Known duplication” section when done.

## Files (expected touch)

```text
crates/fpas-vm/src/vm/execute/io/tui/
  session_app.rs          — NEW: live session helpers + re-entrant run loop
  tv_run.rs
  exec_dialog.rs
  file_dialog.rs
  interactive_loop.rs     — scripted test loop only; live loop moved to session_app.rs
  lifecycle.rs
crates/fpas-vm/src/vm/worker.rs   — live_turbo_vision_app field
crates/fpas-vm/src/vm/shared/tui.rs
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
