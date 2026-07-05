# Done — Single Turbo Vision session per FPAS application

**Status:** [x] Done (2026-07)

## Summary

`Application.Run`, `Application.ExecDialog`, and `Application.RunFileDialog` share one upstream `turbo_vision::app::Application` for the lifetime of Pascal `Application.Open` … `Application.Close`. The live instance lives on the main VM worker (`Worker.live_turbo_vision_app`, `!Send`). Modals opened during `Run` (for example IDE Help → About) reuse menu bar, status line, and terminal state.

## Completed tasks

- [x] **Design** — `session_app.rs`; per-step borrows in `turbo_vision_drive_live_interactive_loop` for re-entrancy
- [x] **Run path** — `tv_run.rs` → `turbo_vision_refresh_live_desktop` + `turbo_vision_drive_live_interactive_loop`
- [x] **ExecDialog** — `turbo_vision_with_live_app` in `exec_dialog.rs`
- [x] **RunFileDialog** — `turbo_vision_with_live_app` in `file_dialog.rs`
- [x] **Headless** — unchanged test stubs; no live TV app in headless mode
- [x] **Lifecycle** — `turbo_vision_shutdown_live_app` on `Close` and `Open` reset
- [x] **Tests** — `cargo test --workspace`; exec dialog, file dialog, IDE tests
- [x] **Docs** — `modals.md`, `lifecycle.md`, `file-dialog.md`, `vm-bridge.md`, `handlers.md`, `session.md`, `testing.md`, `terminal-checklist.md`, `std/tui/README.md`
- [x] **Context** — `00-context.md` updated

## Files touched

```text
crates/fpas-vm/src/vm/execute/io/tui/session_app.rs   — NEW
crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs
crates/fpas-vm/src/vm/execute/io/tui/exec_dialog.rs
crates/fpas-vm/src/vm/execute/io/tui/file_dialog.rs
crates/fpas-vm/src/vm/execute/io/tui/interactive_loop.rs
crates/fpas-vm/src/vm/execute/io/tui/lifecycle.rs
crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs
crates/fpas-vm/src/vm/worker.rs
```

## Verification

```text
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_exec_dialog_test.fpas
cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_file_dialog_test.fpas
cargo run -q -p fpas-cli -- test apps/ide/tests/
```

Manual: run IDE, open Help → About during live session; menu/status remain consistent.

## Notes

- FPAS handle graph remains source of truth for `Create*`; public Pascal API unchanged.
- Headless path still uses queue + custom painter ([03-headless-test-util.md](../03-headless-test-util.md)).
- Follow-up: [03-about-message-box.md](03-about-message-box.md) — IDE About via upstream `message_box`.
