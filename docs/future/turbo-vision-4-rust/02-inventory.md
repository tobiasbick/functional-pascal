# Inventory

Migration status: **complete on branch `turbo-vision-4-rust`** (2026-06-29). This file records what was removed, rewritten, or kept.

## Rust Runtime Areas

Rewritten or reduced during migration:

- [x] `crates/fpas-std/src/tui/` — retained view/widget runtime removed; session, host loop, damage, and geometry helpers kept.
- [x] `crates/fpas-vm/src/vm/execute/io/tui_run.rs` — hosted `Application.Run` plus Turbo Vision branch dispatch.
- [x] `crates/fpas-vm/src/vm/execute/io/tui/` — Turbo Vision facade modules (`controls`, `dialogs`, `file_dialog`, `tv_run`, `host/`, …).
- [x] `crates/fpas-vm/src/vm/shared.rs` — `TurboVisionState` and handle snapshots; old `TuiState.views` removed.

Kept for transition and testing:

- [x] shared console input/event normalization
- [x] test host utilities adapted to headless Turbo Vision and hosted-loop tests
- [x] generic VM callback invocation helpers

## Compiler and Bytecode Areas

- [x] `crates/fpas-sema/src/std_registry/loaded/tui/` — current Turbo Vision public API
- [x] `crates/fpas-compiler/src/compiler/std_calls/tui/` — current lowering
- [x] `crates/fpas-bytecode/src/intrinsic/tui/` — Turbo Vision intrinsics through `CreateTextViewer = 451`
- [x] intrinsic tests updated for current `TuiIntrinsic` set

Target state achieved:

- [x] old public `Host*` Pascal surface removed; internal host-loop intrinsics remain for `Application.Configure`
- [x] intrinsic bridge matches implemented `Application.*` facade only
- [x] intrinsic IDs kept compact; no backward-compatibility gap preservation

## Docs

Current spec under `docs/pascal/std/tui/`:

- [x] `docs/pascal/std/tui/README.md`
- [x] `docs/pascal/std/tui/session.md`
- [x] `docs/pascal/std/tui/app/README.md`
- [x] `docs/pascal/std/tui/app/controls.md`
- [x] `docs/pascal/std/tui/app/modals.md`
- [x] `docs/pascal/std/tui/app/file-dialog.md`
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

Current layout after migration:

- [x] `tests/tui/host/` — hosted-loop and screen-query tests (kept)
- [x] `tests/tui/controls/` — Turbo Vision widget regression tests (12 files)
- [x] removed `tests/tui/scene/`, `frames/`, `menu/`, `modals/` retained-engine-only dirs

Removed assertion categories:

- [x] old local-paint retained scene graph behavior
- [x] `QuerySceneGraph`, `QueryViewState`, `QueryResolvedView`, `QueryFrameRootState`, `QueryMenuBarState`
- [x] old `HostProcessNext` process tags

## Examples and Apps

- [x] `examples/pascal/tui/` — Turbo Vision examples
- [x] TUI entries in `examples/README.md`
- [x] `apps/ide/` — minimal Turbo Vision shell

## Names Removed from Public API

Removed unless noted; replacement uses `Create*`, `AddChild`, `Run`, `OnCommand`, and `RunFileDialog`:

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
