# 04 — Headless tests via turbo-vision `test-util`

**Status:** [ ] Not started · [ ] In progress · [x] Done

**Priority:** Medium–high (large refactor, high long-term payoff)

**Depends on:** [02-single-tv-session.md](02-single-tv-session.md) (live session model).

## Problem (was)

Headless `Application.OpenForTest` + `Run` used a **parallel ASCII painter** (`headless_paint.rs`) that duplicated upstream layout rules. Drift caused false positives/negatives in `AssertScreenCell` tests.

## Outcome

- Workspace `turbo-vision` enables **`test-util`** (`Cargo.toml`).
- `tv_headless_backend.rs` — in-memory `Backend` with queued events.
- `headless_tv_draw.rs` — `HeadlessTvApp` (`Terminal` + `Desktop` + menu/status), upstream `draw` → CRT export via `reconcile.rs` / `Pump`.
- **`headless_paint.rs` removed** — headless repaint uses the same TV view tree as live mode.
- **30/30** `tests/tui/controls/` pass; Rust probe tests document TV buffer cell positions.
- `Worker.headless_tv_app` holds the headless session between `Pump` calls.

## Tasks

- [x] **Dependency** — `test-util` feature on workspace `turbo-vision` dependency.
- [x] **Backend + draw** — `TvHeadlessBackend`, `HeadlessTvApp`, CRT export.
- [x] **Wire Pump** — `turbo_vision_paint_headless_desktop` from `reconcile.rs`.
- [x] **Delete** — `headless_paint.rs`.
- [x] **Regression** — `tests/tui/controls/` (no assertion changes required after layout/export fixes).
- [x] **Docs** — [vm-bridge.md](../../../pascal/std/tui/app/vm-bridge.md), [00-context.md](../00-context.md).

## Follow-up (not blocking)

- [ ] **Input** — Route `TestClickButton` / `TestClickMouse` through `TvHeadlessBackend::push_event` + TV `handle_event`; retire duplicate hit-test in `test_mouse.rs` where upstream covers it.
- [ ] **Shared session** — Optional: attach headless terminal to the same session `Application` as live (coordinate with future session work).

## Files touched

```text
Cargo.toml
crates/fpas-vm/src/vm/execute/io/tui/tv_headless_backend.rs
crates/fpas-vm/src/vm/execute/io/tui/headless_tv_draw.rs
crates/fpas-vm/src/vm/execute/io/tui/chrome_layout.rs
crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs
crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs
crates/fpas-vm/src/vm/worker.rs
```

## Verification

```text
cargo test --workspace
fpas test tests/tui/controls/
```

## Notes

- Headless `MessageBox` still uses `TestSetDialogResult` (same as `ExecDialog`); draw path is now TV-aligned.
- Upstream: `turbo-vision` `test_util` on tag `v2.0.0`; FPAS uses `Terminal::with_backend` rather than `MockTerminal` directly.
