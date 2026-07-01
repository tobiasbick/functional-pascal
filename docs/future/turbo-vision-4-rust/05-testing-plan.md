# Testing Plan

Status: **automated headless coverage complete** on branch `turbo-vision-4-rust`. Manual terminal checks remain optional.

The rewrite replaced old retained-view tests with tests that assert user-visible Turbo Vision behavior.

## Principles

- Prefer FPAS regression tests for public `Std.Tui` behavior.
- Use Rust VM tests for bridge invariants that FPAS cannot express.
- Do not preserve tests for deleted internals.
- Keep headless tests deterministic.
- Do not require a human terminal for CI-style local verification.

## Minimum Headless Capabilities

Implemented through `Application.OpenForTest`, `Application.Test*`, Turbo Vision headless `Application.Run`, and `Application.TestSetFileDialogResult`:

- [x] create test application with fixed width and height
- [x] inject key event
- [x] inject mouse event or command event
- [x] pump one event turn
- [x] query command callback result
- [x] query screen line or screen cell
- [x] close without leaving terminal raw mode active

## Test Categories

### Rust Tests

Covered by workspace tests including:

- [x] handle table validity and Turbo Vision bridge modules
- [x] invalid handle diagnostics
- [x] command callback routing (`tui_spec_links`, compiler/sema TUI suites)
- [x] screen query intrinsics

### FPAS Tests

Covered under `tests/tui/`:

- [x] opening and closing an application (`host/`)
- [x] creating a window (`tui_turbo_vision_window_test.fpas`)
- [x] adding a button and dispatching commands (`tui_turbo_vision_spike_test.fpas`, `run_test.fpas`)
- [x] dialog chrome widgets (static text, memo, text viewer, input line, list box, check box, radio button)
- [x] menu and status line (`tui_turbo_vision_chrome_test.fpas`)
- [x] file dialog accept/cancel (`tui_turbo_vision_file_dialog_test.fpas`)

### Manual Terminal Checks

Optional; not automated:

- [ ] real terminal starts in alternate screen
- [ ] mouse works for buttons and menus
- [ ] window dragging works
- [ ] resize handling works
- [ ] terminal state restores after normal exit
- [ ] terminal state restores after runtime error

## Old Tests Removed

Deleted tests whose only purpose was validating the old engine:

- retained view tree shape
- frame-specific inner viewport clipping
- old menu overlay compositor
- `HostProcessNext` integer process tags
- `QuerySceneGraph` snapshots
- old `ViewId` state query records

## Verification Commands

Baseline after migration:

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -p fpas-cli -- test tests/
cargo run -p fpas-cli -- fmt --check tests/ examples/ apps/
```

Turbo Vision widget subset:

```text
cargo run -p fpas-cli -- test tests/tui/controls/
```

See also [terminal checklist](../../pascal/std/tui/terminal-checklist.md).
