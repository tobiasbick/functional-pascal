# Implementation Phases

Update this file as work progresses. Every phase should leave the repository buildable unless the phase explicitly says it is a short-lived breaking step on the active branch.

## Phase 0: Preparation

- [x] Create local branch `turbo-vision-4-rust`.
- [x] Confirm upstream crate, license, Rust edition, and `crossterm` version.
- [x] Add this future-plan directory.
- [ ] Capture baseline verification on the branch:
  - [ ] `cargo fmt --all -- --check`
  - [ ] `cargo build`
  - [ ] `cargo test --workspace`
  - [ ] `cargo run -p fpas-cli -- test tests/tui/`

## Phase 1: Minimal Dependency Spike

Goal: prove the crate can be added and built without changing public FPAS behavior.

- [x] Add `turbo-vision = "1.3.1"` to workspace dependencies or the owning crate.
- [x] Build the workspace.
- [x] Add a tiny Rust-only smoke test or example behind test-only code if useful. Not added in Phase 1 because this phase only proves dependency resolution; callback behavior is covered by Phase 2.
- [x] Confirm no duplicate `crossterm` version is pulled.
- [x] Record dependency changes in this file.

Notes:

- Verified on 2026-06-29: crates.io reports `turbo-vision` 1.3.1 as the latest/default version and not yanked.
- Upstream `Cargo.toml` for 1.3.1 uses Rust 2024, MIT, library name `turbo_vision`, and `crossterm = "0.29"`.
- Added `turbo-vision = "1.3.1"` to workspace dependencies and `turbo-vision = { workspace = true }` to `fpas-vm`.
- `cargo build` passed after resolving the new dependency.
- `cargo tree -i crossterm` shows a single `crossterm v0.29.0`, shared by `fpas-std` and `turbo-vision`.
- `cargo fmt` and `cargo test --workspace` passed.

Go/no-go:

- [x] `cargo build` passes.
- [x] `cargo tree -i crossterm` shows compatible dependency use.

## Phase 2: FPAS Callback Spike

Goal: prove Turbo Vision commands can call into FPAS.

- [x] Add an internal Turbo Vision command-event bridge that routes `EventType::Command` through the existing FPAS `OnCommand` callback path.
- [x] Add the smallest new TUI intrinsic bridge needed for:
  - [x] create/open application. Uses existing `Application.OpenForTest` for the headless spike.
  - [x] create window or dialog. Implemented `Application.CreateDialog`.
  - [x] create button. Implemented `Application.CreateButton`.
  - [x] register command callback. Implemented `Application.OnCommand`.
  - [x] run or pump application. Implemented one-step `Application.Pump` and the Turbo Vision branch of `Application.Run`.
  - [x] quit application. Implemented `Application.Quit`.
- [x] Add sema registration for only the spike API.
- [x] Add compiler lowering for only the spike API.
- [x] Add VM runtime bridge for only the spike API.
- [x] Add one Rust VM test for command callback behavior.
- [x] Add one FPAS test if headless execution is available.

Notes:

- Added `crates/fpas-vm/src/vm/execute/io/tui/turbo_vision/commands.rs` as the first internal bridge module.
- The Rust VM test constructs `turbo_vision::core::event::Event::command(42)` and verifies that the registered FPAS `OnCommand` handler receives `42`.
- Added the public headless spike API: `TuiDialog`, `TuiButton`, `Application.CreateDialog`, `Application.CreateButton`, `Application.AddChild`, `Application.OnCommand`, `Application.TestClickButton`, `Application.Pump`, and `Application.Quit`.
- Added `tests/tui/controls/tui_turbo_vision_spike_test.fpas`, which creates a dialog/button, queues a button command, dispatches it through `Application.Pump`, and requests quit from the FPAS command handler.
- Added `tests/tui/controls/tui_turbo_vision_run_test.fpas`, which uses `Application.Run` over the Turbo Vision headless path and verifies that the queued button command reaches FPAS.
- `Application.Run` now detects Turbo Vision handles. In headless sessions it drains queued commands without a terminal; in terminal sessions it builds a short-lived upstream `turbo_vision::app::Application` from the Send-safe FPAS handle metadata and calls upstream `run()`.
- `Application.Open` now opens a logical FPAS session without acquiring retained-engine terminal state. The old retained hosted loop acquires terminal state in its own `Application.Run` path; the Turbo Vision path leaves terminal initialization to upstream Turbo Vision.
- Added `tui_session_open_deferred_does_not_acquire_terminal_writer` to guard the deferred-open terminal boundary.
- Updated `docs/pascal/std/tui/app/README.md` for the implemented spike API only.

Go/no-go:

- [x] Button command reaches FPAS callback.
- [x] Callback can request quit.
- [x] Test can run without manual terminal interaction.
- [x] Production `Application.Run` can use Turbo Vision's terminal application loop after Turbo Vision handles are created.
- [x] Production `Application.Open` no longer acquires the old retained-engine terminal session before Turbo Vision runs.

## Phase 3: Remove Old Public Host API

Goal: stop expanding the old retained engine.

- [x] Remove or de-register old `Application.Host*` public calls.
- [x] Remove old query calls tied to retained internals.
- [x] Remove old `TuiIntrinsic` variants that are no longer reachable.
- [x] Update compiler tests for the removed public Host API.
- [x] Update bytecode tests after unreachable intrinsic variants are removed.
- [x] Update diagnostics to mention the new API when old symbols are absent.

Temporary breakage allowed only inside this phase if fixed before moving on.

Notes:

- De-registered the old public `Application.Host*` Sema modules and removed the corresponding compiler lowering modules.
- Replaced Sema tests with negative coverage showing old Host symbols are absent.
- Replaced active compiler TUI tests with current session, Turbo Vision command-run, and old-symbol negative coverage.
- Deleted FPAS TUI regression tests that only exercised the removed retained Host API.
- Rewrote `docs/pascal/std/tui/` app/session pages so current docs describe implemented behavior only.
- De-registered retained `Application.QueryView*`, `Application.QuerySceneGraph`, frame queries, modal depth/focus queries, and `Application.ShowFramedDialog`.
- Kept only headless screen queries (`QueryScreenSize`, `QueryScreenLine`, `QueryScreenCell`) as current public test surface.
- Removed the now-unreachable retained query, frame, framed-dialog, modal-depth, and focused-view `TuiIntrinsic` variants.
- Replaced retained query bytecode tests with screen-query coverage and removed direct frame/framed-dialog intrinsic tests.
- Added Sema diagnostics for removed old TUI symbols so unknown `Application.Host*`, retained query, modal, and framed-dialog calls point users at the current Turbo Vision facade.
- Next: replace retained-engine internals with Turbo Vision-backed application, dialog, command, event, and widget modules.

## Phase 4: Replace Runtime Engine

Goal: make Turbo Vision the only production TUI engine.

- [x] Remove old frame/window manager.
- [x] Remove old retained view registry.
- [x] Remove old menu widget implementation.
- [x] Remove old retained control-widget intrinsic and event dispatch paths.
- [x] Remove remaining old control-widget storage/rendering where Turbo Vision supplies replacements.
- [x] Keep or adapt only reusable terminal/test abstractions.
- [x] Keep files below project size expectations by grouping by concern.

Notes:

- Removed the unreachable old retained control-widget `TuiIntrinsic` variants for labels, buttons, input lines, check boxes, radio groups, list boxes, scroll bars, scroll views, and memos.
- Removed the VM-side retained control intrinsic bridge and retained control input dispatcher. Host key, mouse, and paste processing now skip the old control-model dispatch path.
- Verified no remaining code references to the removed retained control intrinsic names or old retained control dispatch functions.
- Removed retained control widget storage/rendering types from `fpas-std`, including label, button, input line, check box, radio group, list box, standalone scroll bar, scroll view, and memo widgets.
- Kept only `ScrollBarStyle` because legacy frame chrome still uses the style values internally. The standalone retained scroll bar widget is gone.
- Removed old retained control symbol names from the known `Std.Tui` symbol list.
- Removed the retained menu-bar widget, popup, parser, event dispatcher, bytecode intrinsics, and old known-symbol entries. Current `docs/pascal/` does not describe the removed retained menu API; planned future `MenuBar` work remains in Phase 5.
- Removed the retained frame/window runtime from `fpas-std` and `fpas-vm`: deleted `widget/frame/`, frame-only `scroll/`, `ViewWidget::Frame`, frame-root registry state, frame command dispatch (zoom/restore/close), and frame symbol entries from `STD_TUI_SYMBOLS`.
- Moved `activate_next_root_excluding` into `view/activation.rs` so `NextWindow` still cycles retained roots without the frame module.
- Next: continue Turbo Vision-backed module layout from the phase target tree.
- Removed the VM `views/*` bytecode bridge and retained-view/modal/status-bar intrinsic variants. Pruned `STD_TUI_SYMBOLS` to the current public Turbo Vision facade plus host-loop symbols.
- Removed `fpas-std/tui/view/*`, `widget/*`, and `modal/*` plus `TuiState.views`, `view_paints`, `view_widgets`, `view_commands`, and `modals`. The hosted loop now uses global `OnPaint` only; event handlers request full-frame redraw hints. Removed `Std.Test.AssertViewRect`.
- Removed retained-widget CRT paint helpers (`fill_rect_crt`, `write_text_at_crt`, `write_char_at_crt`) and unused handler stack decoders. Renamed `turbo_vision/widgets.rs` to `controls.rs`.
- Flattened `turbo_vision/` into themed VM modules (`handles`, `dialogs`, `controls`, `callbacks`, `commands`, `tv_run`, `events`, `testing`) and renamed `test_host.rs` to `testing.rs`. See `docs/pascal/std/tui/app/vm-bridge.md`.

Expected layout under `crates/fpas-vm/src/vm/execute/io/tui/` (implemented; `tv_run.rs` holds Turbo Vision `Application.Run` because `application.rs` is the Pascal session lifecycle):

```text
  application.rs     -- Application lifecycle and configuration
  callbacks.rs       -- FPAS callback invocation from Turbo Vision commands
  commands.rs        -- command queue, Pump, Quit, TestClickButton
  controls.rs        -- button construction and AddChild
  dialogs.rs         -- CreateDialog
  events.rs          -- headless test event injection
  handles.rs         -- host-owned handle table
  testing.rs         -- OpenForTest, TestPump*, CloseForTest
  tv_run.rs          -- Turbo Vision Application.Run (terminal + headless)
  host/              -- hosted global-handler loop
```

## Phase 5: Build the Real API

Implement in this order:

- [x] `Application` — session open/close/run, Turbo Vision pump, and `OnCommand` from Phase 2–3 spike.
- [x] `Rect`, `Point`, `Size` — value records registered in Sema (`Point` added 2026-06-29).
- [x] `Command` — `Command.Accept`, `Command.Cancel`, `Command.Close`, `Command.Quit` integer constants (2026-06-29; `Accept` not `Ok` — `Ok` is a keyword).
- [x] `Window` — `Application.CreateWindow`, `Application.AddWindow`, and `Application.AddChild` parent support (2026-06-29).
- [x] `Dialog` — `Dialog` handle type (renamed from spike `TuiDialog`, 2026-06-29).
- [x] `Button` — `Button` handle type (renamed from spike `TuiButton`, 2026-06-29).
- [x] `StaticText` — `Application.CreateStaticText`, `StaticText` handle type, and `Application.AddChild` child support (2026-07-01).
- [x] `InputLine` — `Application.CreateInputLine`, `InputLine` handle type, max-length validation, and `Application.AddChild` child support (2026-07-01).
- [x] `MenuBar` — `MenuBar`, `MenuBarItem`, `Application.CreateMenuBar`, and `Application.SetMenuBar` with one command entry per top-level menu item (2026-07-01).
- [x] `StatusLine` — `StatusLine`, `StatusItem`, `Application.CreateStatusLine`, and `Application.SetStatusLine` (2026-07-01).
- [x] `ListBox` — `Application.CreateListBox`, `ListBox` handle type, string-array items, select command id, and `Application.AddChild` child support (2026-07-01).
- [x] `CheckBox` — `Application.CreateCheckBox`, `CheckBox` handle type, initial checked state, and `Application.AddChild` child support (2026-07-01).
- [x] `RadioButton` — `Application.CreateRadioButton`, `RadioButton` handle type, group id, selected state, and `Application.AddChild` child support (2026-07-01).
- [x] `Memo` — `Application.CreateMemo`, `Memo` handle type, multi-line initial text, and `Application.AddChild` child support (2026-07-01).
- [x] **File dialog** — `Application.RunFileDialog`, `Application.TestSetFileDialogResult`, `Option of string` result, headless test override (2026-06-29).
- [x] **`TextViewer`** — `Application.CreateTextViewer`, `TextViewer` handle type, read-only multi-line initial text, and `Application.AddChild` child support (2026-06-29).

Each item requires:

- [x] sema registration
- [x] compiler lowering
- [x] bytecode intrinsic if needed
- [x] VM/runtime implementation
- [x] current docs under `docs/pascal/std/tui/`
- [x] focused tests

## Phase 6: Migrate Examples and IDE

- [x] Rewrite `examples/pascal/tui/`.
- [x] Update `examples/README.md`.
- [x] Rewrite `apps/ide/`.
- [x] Remove examples that only demonstrate deleted internals.
- [x] Run FPAS formatter checks on changed FPAS sources.

Notes:

- Removed retained-view examples that depended on deleted `Application.Host*`, retained frame, menu, modal, and view query APIs.
- Kept `minimal_application.fpas` as the current hosted-loop example and changed it to use `Application.Quit`.
- Added current Turbo Vision dialog and window examples using `Dialog`, `Window`, `Button`, `StaticText`, `Application.AddChild`, `Application.AddWindow`, `Application.OnCommand`, and `Command.Quit`.
- Rewrote `apps/ide` as a minimal Turbo Vision shell over the currently implemented API. Rich dialog chrome remains blocked on later Phase 5 widgets.
- Added/updated IDE tests for command constants, About command state, shell exit command dispatch, status text, and theme constants.

## Phase 7: Final Documentation

- [x] Replace current `docs/pascal/std/tui/` with implemented API docs (2026-06-29).
- [x] Remove stale retained-engine pages (`app/frames.md`, `app/views.md`).
- [x] Update `docs/pascal/std/README.md`.
- [x] Rust `///` docs already link to current TUI paths; no stale `frames`/`views` links found.
- [x] Keep future-only notes in this directory.

## Phase 8: Verification and Cleanup

- [x] `cargo fmt` (2026-06-29).
- [x] `cargo build`.
- [x] `cargo test --workspace`.
- [x] `cargo run -p fpas-cli -- test tests/` — 311 passed, 1 skipped.
- [x] `fpas fmt --check tests/ examples/ apps/` — three unrelated `examples/pascal/` files reformatted.
- [x] Remove dead modules and unused imports — no additional TUI dead modules found; hosted-loop and `ViewRect`/`DamageRegion` internals remain for the transition path.
- [x] Confirm no `.github/workflows/` or automation config was added.
