# Inventory

Use this file before deleting or moving code. Check each area and update the notes when implementation starts.

## Rust Runtime Areas

Likely removal or heavy rewrite:

- [ ] `crates/fpas-std/src/tui/`
- [ ] `crates/fpas-vm/src/vm/execute/io/tui_run.rs`
- [ ] `crates/fpas-vm/src/vm/execute/io/tui/`
- [ ] `crates/fpas-vm/src/vm/shared.rs` fields for old `TuiState`

Keep only if still needed after the spike:

- [ ] shared console input/event normalization
- [ ] test host utilities that can be adapted to Turbo Vision
- [ ] generic VM callback invocation helpers

## Compiler and Bytecode Areas

Likely rewrite:

- [ ] `crates/fpas-sema/src/std_registry/loaded/tui/`
- [ ] `crates/fpas-compiler/src/compiler/std_calls/tui/`
- [ ] `crates/fpas-bytecode/src/intrinsic/tui/`
- [ ] intrinsic tests that assert old `TuiIntrinsic` variants

Target state:

- [ ] remove old `Host*` intrinsic variants
- [ ] add only the intrinsic bridge needed by the new API
- [ ] keep intrinsic IDs compact; no backward-compatibility gap preservation is required

## Docs

Current spec (replaced during Turbo Vision migration):

- [x] `docs/pascal/std/tui/README.md`
- [x] `docs/pascal/std/tui/session.md`
- [x] `docs/pascal/std/tui/app/README.md`
- [x] `docs/pascal/std/tui/app/controls.md`
- [x] `docs/pascal/std/tui/app/modals.md`
- [x] `docs/pascal/std/tui/app/handlers.md`
- [x] `docs/pascal/std/tui/app/testing.md`
- [x] `docs/pascal/std/tui/app/types.md`
- [x] `docs/pascal/std/tui/app/vm-bridge.md`
- [x] `docs/pascal/std/tui/terminal-checklist.md`
- [x] `docs/pascal/std/README.md` (TUI hub row and quick example)

Removed stale retained-engine stubs:

- [x] `docs/pascal/std/tui/app/views.md` (deleted)
- [x] `docs/pascal/std/tui/app/frames.md` (deleted)

## FPAS Tests

Likely delete or rewrite:

- [ ] `tests/tui/host/`
- [ ] `tests/tui/scene/`
- [ ] `tests/tui/controls/`
- [ ] `tests/tui/frames/`
- [ ] `tests/tui/menu/`
- [ ] `tests/tui/modals/`

Do not preserve assertions that exist only for old retained-view internals, including:

- old local-paint retained scene graph behavior
- `QuerySceneGraph`
- `QueryViewState`
- `QueryResolvedView`
- `QueryFrameRootState`
- `QueryMenuBarState`
- old `HostProcessNext` process tags

## Examples and Apps

Likely rewrite:

- [ ] `examples/pascal/tui/`
- [ ] TUI entries in `examples/README.md`
- [ ] `apps/ide/`

Examples should demonstrate the new FPAS API, not preserve old host bridge names.

## Names to Remove

Remove these from the public API unless a later implementation note explicitly keeps one:

- `Application.HostRegisterView`
- `Application.HostPushChildView`
- `Application.HostSetViewParent`
- `Application.HostRegisterOnViewPaint`
- `Application.HostCreate*View`
- `Application.HostCreateFrameView`
- `Application.ShowFramedDialog`
- `Application.HostProcessNext`
- `Application.HostDispatchRedraw`
- `Application.QuerySceneGraph`
- `Application.QueryViewState`
- `Application.QueryResolvedView`
- `Application.QueryFrameRootState`

The replacement API should use ordinary names such as `CreateWindow`, `CreateDialog`, `CreateButton`, `AddChild`, `Run`, and `OnCommand`.
