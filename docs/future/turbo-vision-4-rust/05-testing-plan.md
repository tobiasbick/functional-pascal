# Testing Plan

Status: **automated headless coverage complete** on branch `turbo-vision-4-rust` (28 widget/control
tests under `tests/tui/controls/` as of 2026-07-02). Manual terminal checks remain optional.

The rewrite replaced old retained-view tests with tests that assert user-visible Turbo Vision behavior.

## Principles

- Prefer FPAS regression tests for public `Std.Tui` behavior.
- Use Rust VM tests for bridge invariants FPAS cannot express (input-line cell, scripted interactive loop, query bounds).
- Do not preserve tests for deleted internals.
- Keep headless tests deterministic.
- Do not require a human terminal for CI-style local verification.

## Minimum headless capabilities

Implemented via `Application.OpenForTest`, `Application.Test*`, Turbo Vision `Application.Run` /
`Application.Pump`, and test overrides (`TestSetFileDialogResult`, `TestSetDialogResult`):

- [x] fixed-size test application
- [x] inject key / mouse / command events
- [x] pump or run one or more event turns
- [x] query command callback results
- [x] query screen content via `Std.Test.AssertScreenLine` / `AssertScreenCell` on the CRT back buffer
- [x] modal dialog close command and field read-back (`InputText`, `Checked`)
- [x] live tree mutations visible after reconcile (`live_tree`, `live_dialog` tests)
- [x] runtime property setters (`set_*` tests)
- [x] close without leaving terminal raw mode active

## Test categories

### Rust tests (`crates/fpas-vm/src/tests/`)

- [x] Turbo Vision command callback routing
- [x] screen assertion helpers and out-of-range diagnostics (`Std.Test` + `Console::query_screen_*`)
- [x] scripted interactive loop (command + unhandled key)
- [x] handle / bridge module invariants

### FPAS tests (`tests/tui/controls/`)

**Turbo Vision controls** (28 files):

| Theme | Examples |
| --- | --- |
| Spike / run | `tui_turbo_vision_spike_test.fpas`, `tui_turbo_vision_run_test.fpas`, `tui_run_path_test.fpas` |
| Chrome | `chrome_test.fpas`, `menu_test.fpas` |
| Widgets | `window`, `static_text`, `memo`, `text_viewer`, `input_line`, `list_box`, `check_box`, `radio_button` |
| File / modal | `file_dialog_test.fpas`, `exec_dialog_test.fpas`, `checked_test.fpas` |
| Live reconcile | `live_tree_test.fpas`, `live_dialog_test.fpas` |
| Runtime setters | `set_text`, `set_checked`, `set_items`, `set_title`, `set_menus`, `set_status_items` |
| Command map | `reserved_command_test.fpas` |

### Manual terminal checks

Optional; not automated — see `docs/pascal/std/tui/terminal-checklist.md`:

- [ ] alternate screen and restored terminal on exit
- [ ] mouse on buttons and menus
- [ ] window drag / resize
- [ ] runtime error still restores terminal
- [ ] `exec_dialog.fpas` and `runtime_setters.fpas` behave as expected interactively

## Removed test categories

Tests deleted because they only validated the old retained engine:

- retained view tree / scene graph
- frame inner viewport clipping
- old menu overlay compositor
- `HostProcessNext` process tags
- `QuerySceneGraph`, `QueryViewState`, retained modal depth queries
- hosted canvas loop tests (`tests/tui/host/`)
- public `Application.QueryScreen*` (screen checks use `Std.Test.AssertScreenLine` / `AssertScreenCell`)

## Verification commands

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -p fpas-cli -- test tests/
cargo run -p fpas-cli -- fmt --check tests/ examples/ apps/
```

Turbo Vision subset:

```text
cargo run -p fpas-cli -- test tests/tui/controls/
```

Post-migration setter subset:

```text
cargo run -p fpas-cli -- test tests/tui/controls/ --filter set_
```
