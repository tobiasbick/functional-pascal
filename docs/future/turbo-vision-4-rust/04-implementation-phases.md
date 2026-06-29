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

- [ ] Add `turbo-vision = "1.3.1"` to workspace dependencies or the owning crate.
- [ ] Build the workspace.
- [ ] Add a tiny Rust-only smoke test or example behind test-only code if useful.
- [ ] Confirm no duplicate `crossterm` version is pulled.
- [ ] Record dependency changes in this file.

Go/no-go:

- [ ] `cargo build` passes.
- [ ] `cargo tree -i crossterm` shows compatible dependency use.

## Phase 2: FPAS Callback Spike

Goal: prove Turbo Vision commands can call into FPAS.

- [ ] Add the smallest new TUI intrinsic bridge needed for:
  - [ ] create/open application
  - [ ] create window or dialog
  - [ ] create button
  - [ ] register command callback
  - [ ] run or pump application
  - [ ] quit application
- [ ] Add sema registration for only the spike API.
- [ ] Add compiler lowering for only the spike API.
- [ ] Add VM runtime bridge for only the spike API.
- [ ] Add one Rust VM test for command callback behavior.
- [ ] Add one FPAS test if headless execution is available.

Go/no-go:

- [ ] Button command reaches FPAS callback.
- [ ] Callback can request quit.
- [ ] Test can run without manual terminal interaction.

## Phase 3: Remove Old Public Host API

Goal: stop expanding the old retained engine.

- [ ] Remove or de-register old `Application.Host*` public calls.
- [ ] Remove old query calls tied to retained internals.
- [ ] Remove old `TuiIntrinsic` variants that are no longer reachable.
- [ ] Update compiler and bytecode tests.
- [ ] Update diagnostics to mention the new API when old symbols are absent.

Temporary breakage allowed only inside this phase if fixed before moving on.

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
