# IDE implementation phases

Complete phases in order. A phase is complete only when its implementation,
tests, current documentation, and listed verification pass. Record evidence in
the completion log; do not mark work complete based only on source inspection.

## Phase status

- [ ] Phase 1 — captured process execution
- [ ] Phase 2 — controlled multiline `TextArea`
- [ ] Phase 3 — replacement application skeleton
- [ ] Phase 4 — document lifecycle
- [ ] Phase 5 — Check and Run integration
- [ ] Phase 6 — terminal polish and promotion

## Phase 1 — captured process execution

Goal: provide the smallest host API needed to invoke the active compiler and
show its output.

Implementation:

- Add `ProcessOutput`, `CurrentExecutable`, and `RunCapture` to `Std.Proc` across
  sema registration, compiler lowering, bytecode, runtime, and VM dispatch.
- Put new Rust code in focused `proc/` modules if the owning file approaches 400
  lines; do not grow a mixed process file beyond 500 lines.
- `RunCapture` returns exit code, stdout, and stderr without writing child output
  to the IDE terminal.
- Update `docs/pascal/std/host/proc.md` only when the API exists.

Tests:

- Rust tests for current-executable lookup, successful capture, non-zero exit,
  and spawn failure.
- FPAS regression invoking the current `fpas --version` and asserting captured
  output.

Verify:

```text
cargo fmt
cargo build --workspace
cargo test --workspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/stdlib/proc/
```

## Phase 2 — controlled multiline `TextArea`

Goal: add a generic multiline editor control without IDE-specific behavior.

Implementation:

- Add the `TextArea` element and builder, `TextAreaChanged` message/helper,
  validation, measurement, arrangement, painting, focus discovery, keyboard
  routing, and pointer caret placement.
- Support the keys listed in [product.md](product.md#editing-behavior).
- Keep caret and offset model-owned; routing proposes complete next values.
- Update the split `Std.Tui` docs when behavior exists.

Tests under `tests/stdlib/tui/`:

- multiline insert/delete and newline handling;
- arrow, Home/End, PageUp/PageDown, and Tab behavior;
- automatic caret visibility and explicit scroll state;
- focused/unfocused snapshots and clipping;
- pointer caret placement;
- invalid caret/offset diagnostics.

Verify:

```text
cargo fmt
cargo build --workspace
cargo test --workspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/stdlib/tui/
```

## Phase 3 — replacement application skeleton

Goal: remove the unsupported IDE source and render the fixed headless screen
from the new module layout.

Implementation:

- Apply the deletion map in [replacement-inventory.md](replacement-inventory.md).
- Create the target modules from [architecture.md](architecture.md#target-source-layout).
- Implement empty/argument-loaded document state, stable typed IDs/actions,
  `Update`, and the fixed `View`.
- Render the flat Open/Save/Check/Run/Exit command bar, editor, message area,
  and status line. Commands may report `not implemented` during this phase.
- Rewrite `ide-core.fpasprj` exports and keep `ide.fpasprj` as a thin program.

Tests:

- Initial model with and without a path argument.
- Headless surface assertions for 80×25 and one smaller supported size.
- Action dispatch updates the correct model field without file/process effects.
- Create `tests/ide/ide-tests.fpasprj`, add IDE tests and the `ide-core`
  dependency to the central test project, and add a targeted Cargo suite shard.

Verify:

```text
cargo run -q -p fpas-cli -- fmt --check apps/ide tests/ide
cargo run -q -p fpas-cli -- check --std-lib lib apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo test -p fpas-cli fpas_suite_ide
```

## Phase 4 — document lifecycle

Goal: make Open, Save, Save As, and dirty protection complete.

Implementation:

- Implement UTF-8 read/write through `Std.Fs` in `Ide.Document.Io`.
- Add Open and Save-path dialogs using `Input`, labels, and buttons.
- Add Save/Discard/Cancel protection before Open and Exit.
- Derive dirty state from `Text <> SavedText`.
- Keep the current document unchanged on every failed read/write.
- Use `.temp-data/ide/` for test files.

Tests:

- open success/failure, save success/failure, untitled Save, and dirty derivation;
- each dirty-confirm branch;
- an end-to-end headless open-edit-save slice.

Verify with the Phase 3 commands plus the full `fpas test tests/` suite.

## Phase 5 — Check and Run integration

Goal: execute the current compiler against the saved document and present its
result.

Implementation:

- Resolve the command with `Std.Proc.CurrentExecutable`.
- Save before Check or Run; abort the process command if saving fails.
- Invoke `check <path>` or `run <path>` with `RunCapture`.
- Format exit code, stdout, and stderr deterministically in the message area.
- Keep synchronous execution and non-interactive child programs as documented
  constraints.

Tests:

- argument construction for Check and Run;
- success, compiler diagnostic, runtime diagnostic, and process-start failure;
- pure check/run argument construction and output formatting;
- one repository integration program that invokes the current executable and
  verifies the resulting IDE message state.

Verify with targeted IDE tests, `cargo test --workspace`, and one manual run on
a temporary valid and invalid `.fpas` file.

## Phase 6 — terminal polish and promotion

Goal: make the replacement supported and remove planning-only state.

Implementation:

- Verify keyboard shortcuts, pointer activation, resize behavior, dialogs,
  caret visibility, long diagnostics, and terminal restoration manually.
- Update the repository references listed in the replacement inventory.
- Replace status-only IDE docs with current run and usage instructions.
- Confirm no removed IDE unit names remain.
- Remove `docs/future/ide/` and its row in `docs/future/README.md` only after the
  implementation, current docs, and tests contain all durable information.

Final verification:

```text
cargo fmt
cargo build --workspace
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check apps/ide tests/ide
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test --std-lib lib tests/
rg -n "Ide\.(Shell|Ui|Dialog|Workspace)" apps docs examples crates tests
git diff --check
```

## Completion log

Add one row per completed phase.

| Phase | Date | Evidence | Notes |
| --- | --- | --- | --- |
| — | — | — | No phase has started. |
