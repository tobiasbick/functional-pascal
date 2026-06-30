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
- [ ] Update diagnostics to mention the new API when old symbols are absent.

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
- Next: add old-symbol diagnostics that point users at the current Turbo Vision facade.

## Phase 4: Replace Runtime Engine

Goal: make Turbo Vision the only production TUI engine.

- [ ] Remove old retained view registry.
- [ ] Remove old frame/window manager.
- [ ] Remove old menu widget implementation.
- [ ] Remove old control widgets where Turbo Vision supplies replacements.
- [ ] Keep or adapt only reusable terminal/test abstractions.
- [ ] Keep files below project size expectations by grouping by concern.

Expected new layout, adjust before editing if implementation reveals better boundaries:

```text
crates/fpas-vm/src/vm/execute/io/tui/
  application.rs     -- Application lifecycle and handle lookup
  callbacks.rs       -- FPAS callback invocation from Turbo Vision commands
  commands.rs        -- command IDs and conversion
  controls.rs        -- widget construction bridge
  dialogs.rs         -- modal dialog bridge
  events.rs          -- event conversion and injection
  handles.rs         -- host-owned handle table
  testing.rs         -- headless/test-only bridge
```

## Phase 5: Build the Real API

Implement in this order:

- [ ] `Application`
- [ ] `Rect`, `Point`, `Size`
- [ ] `Command`
- [ ] `Window`
- [ ] `Dialog`
- [ ] `Button`
- [ ] `StaticText`
- [ ] `InputLine`
- [ ] `MenuBar`
- [ ] `StatusLine`
- [ ] `ListBox`
- [ ] `CheckBox`
- [ ] `RadioButton`
- [ ] `Memo` or `TextViewer`
- [ ] file dialog only after the core app and tests are stable

Each item requires:

- [ ] sema registration
- [ ] compiler lowering
- [ ] bytecode intrinsic if needed
- [ ] VM/runtime implementation
- [ ] current docs under `docs/pascal/std/tui/`
- [ ] focused tests

## Phase 6: Migrate Examples and IDE

- [ ] Rewrite `examples/pascal/tui/`.
- [ ] Update `examples/README.md`.
- [ ] Rewrite `apps/ide/`.
- [ ] Remove examples that only demonstrate deleted internals.
- [ ] Run FPAS formatter checks on changed FPAS sources.

## Phase 7: Final Documentation

- [ ] Replace current `docs/pascal/std/tui/` with implemented API docs.
- [ ] Remove stale retained-engine pages.
- [ ] Update `docs/pascal/std/README.md`.
- [ ] Update Rust `///` docs that link to old TUI paths.
- [ ] Keep future-only notes in this directory.

## Phase 8: Verification and Cleanup

- [ ] `cargo fmt`
- [ ] `cargo build`
- [ ] `cargo test --workspace`
- [ ] `cargo run -p fpas-cli -- test tests/`
- [ ] `fpas fmt --check tests/ examples/ apps/` when FPAS files changed
- [ ] Remove dead modules and unused imports.
- [ ] Confirm no `.github/workflows/` or automation config was added.
