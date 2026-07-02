# Inventory

Migration status: **complete on branch `turbo-vision-4-rust`** (core migration 2026-06-29; post-migration
through 2026-07-02). This file records what was removed, rewritten, or kept. Open items are in
[07-post-migration-improvements.md](07-post-migration-improvements.md).

## Rust Runtime Areas

Rewritten or reduced during migration:

- [x] `crates/fpas-std/src/tui/` — retained view/widget runtime removed; session, host loop, damage, and geometry helpers kept.
- [x] `crates/fpas-vm/src/vm/execute/io/tui_run.rs` — Turbo Vision `Application.Run` only (hosted canvas branch removed in Track 08).
- [x] `crates/fpas-vm/src/vm/execute/io/tui/` — Turbo Vision facade (`controls`, `dialogs`, `windows`,
  `navigation`, `exec_dialog`, `file_dialog`, `reconcile`, `headless_paint`, `bridged_check_box`,
  `interactive_loop`, `tv_run`, …)
- [x] `crates/fpas-vm/src/vm/shared.rs` — `TurboVisionState`, handle snapshots, `TurboVisionBoolCell` /
  `TurboVisionInputTextCell`; old `TuiState.views` removed

Kept for transition and testing:

- [x] shared console input/event normalization
- [x] test host utilities adapted to headless Turbo Vision (`OpenForTest`, `Std.Test` screen asserts)
- [x] generic VM callback invocation helpers

## Compiler and Bytecode Areas

- [x] `crates/fpas-sema/src/std_registry/loaded/tui/` — current Turbo Vision public API
- [x] `crates/fpas-compiler/src/compiler/std_calls/tui/` — current lowering
- [x] `crates/fpas-bytecode/src/intrinsic/tui/` — Turbo Vision widget intrinsics through `Checked = 464`
  (`widgets.inc`; session/query intrinsics in sibling variant files)
- [x] intrinsic tests updated for current `TuiIntrinsic` set

Target state achieved:

- [x] old public `Host*` Pascal surface removed; TUI hosted canvas loop and its intrinsics removed (Track 08)
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

- [x] `tests/tui/controls/` — Turbo Vision widget and post-migration regression tests (28 files)
- [x] removed `tests/tui/host/` (hosted canvas loop tests; Track 08)
- [x] removed `tests/tui/scene/`, `frames/`, `menu/`, `modals/` retained-engine-only dirs

Removed assertion categories:

- [x] old local-paint retained scene graph behavior
- [x] `QuerySceneGraph`, `QueryViewState`, `QueryResolvedView`, `QueryFrameRootState`, `QueryMenuBarState`
- [x] old `HostProcessNext` process tags

## Examples and Apps

- [x] `examples/pascal/tui/` — Turbo Vision examples (`turbo_vision_dialog`, `turbo_vision_window`,
  `exec_dialog`, `runtime_setters`)
- [x] `examples/math/mandelbrot/mandelbrot.fpasprj` — terminal explorer on `Std.Console` (not `Std.Tui`)
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
