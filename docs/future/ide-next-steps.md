# Future: IDE next steps

> Status snapshot: 2026-07-25. Scrollable message output and diagnostic source
> navigation are implemented. The next decision is project/workspace startup.

## Purpose

Continue the small terminal IDE in `apps/ide` without restoring the abandoned
IDE architecture and without introducing a tiled window manager. Compiler
output is now usable in the single-document workflow. Decide the required
startup model before expanding the document or project state.

The implemented behavior is documented in:

- [`docs/pascal/apps/ide.md`](../pascal/apps/ide.md)
- [`apps/ide/README.md`](../../apps/ide/README.md)

Those files are authoritative for current behavior. Keep proposed behavior in
this document until it is implemented and tested.

## Current implementation

The IDE is a fixed, single-document `Std.Tui` application:

- it opens or starts with one UTF-8 `.fpas` source;
- it edits the source with a controlled multiline `TextArea`;
- it supports Open, Save, Check, Run, and Exit;
- it protects dirty documents with Save, Discard, and Cancel;
- it provides F2, F9, Ctrl+F9, and Alt+X shortcuts;
- Check and Run save first and invoke the current `fpas` executable
  synchronously;
- captured stdout and stderr are normalized and stored in `MessageText`;
- the complete message text is displayed through a bounded three-line scroll
  viewport with model-owned offset;
- Check diagnostics for the open document are parsed, selected, and navigable
  with Enter;
- dialogs, pointer activation, resize handling, and terminal restoration have
  headless or lifecycle coverage.

Current ownership is intentionally small:

```text
apps/ide/
  ├── ide-core.fpasprj          — reusable library for the application and tests
  ├── ide.fpasprj               — executable project
  ├── ide.fpasworkspace         — workspace connecting both projects
  └── src/
      ├── main.fpas             — argument parsing and TuiApplication.Run
      ├── app/
      │   ├── actions.fpas      — stable control and action identities
      │   ├── document_flow.fpas— document lifecycle and process dispatch
      │   ├── model.fpas        — root immutable application state
      │   ├── update.fpas       — TuiMsg routing and shortcuts
      │   └── view.fpas         — fixed IDE surface and modal dialogs
      ├── document/
      │   ├── io.fpas           — UTF-8 file operations
      │   └── model.fpas        — text, caret, viewport, path, and dirty state
      ├── diagnostic/
      │   ├── model.fpas        — parsed source navigation data
      │   └── parser.fpas       — captured diagnostic header parsing
      └── process/
          └── runner.fpas       — Check/Run invocation and output formatting
```

The regression suite is in `tests/ide/ide-tests.fpasprj`. It covers the model,
surface, document I/O, dirty-document decisions, dialogs, pointer interaction,
shortcuts, diagnostic parsing and navigation, process argument/format behavior,
process integration, and the document lifecycle.

## Known limits

Deliberate limits are:

- one open document only;
- no `.fpasprj` or `.fpasworkspace` startup target;
- synchronous Check and Run;
- no stdin connection for child programs;
- no help system;
- no tiled windows or tiled window manager.

Treat each remaining limit as a separate planned slice.

## Completed slice: scrollable message output

> Completed 2026-07-25.

### Outcome

The Messages window is a bounded multiline viewport. The latest complete
Check/Run report remains in the model and can be scrolled vertically without
changing the editor document.

### Reuse boundary

The implementation reuses the controlled `TuiElement.Scroll` API:

```text
MakeScroll(Id, Offset, ChangeAction, Child)
TuiMsg.ScrollChanged(Source, Action, Offset)
```

`Std.Tui` already routes scroll keyboard input and pointer focus. A focused
layout audit showed that the old API could not bound a scroll viewport:
`Scroll` inherited its complete content minimum, and `TuiLayoutSettings` had no
explicit fixed height. The generic correction added a one-cell `Scroll`
minimum and `TuiLayoutSettings.WithFixedHeight`.

### Implemented file layout

```text
apps/ide/src/app/
  ├── actions.fpas       — MODIFY: message-scroll control and action identities
  ├── model.fpas         — MODIFY: controlled message offset
  ├── document_flow.fpas — MODIFY: preserve or reset message offset
  ├── update.fpas        — MODIFY: accept the matching ScrollChanged message
  └── view.fpas          — MODIFY: bounded Messages window containing Scroll

lib/Std/Tui/Layout/
  ├── LayoutSettings.fpas — MODIFY: WithFixedHeight
  ├── Measure.fpas        — MODIFY: bounded layouts and shrinkable Scroll
  └── Arrange.fpas        — MODIFY: fixed-height wrappers do not grow

tests/ide/
  ├── ide_model_test.fpas               — MODIFY: initial message offset
  ├── ide_update_test.fpas              — MODIFY: scroll routing and reset
  ├── ide_surface_test.fpas             — MODIFY: multiline viewport and clipping
  ├── ide_interaction_test.fpas         — MODIFY: focus, scrolling, and resize
  └── ide_process_integration_test.fpas — MODIFY: process-result offset reset

tests/stdlib/tui/
  ├── layout_values_test.fpas                 — MODIFY: fixed-height settings
  ├── scroll_layout_test.fpas                 — MODIFY: bounded Scroll measurement
  └── fixed_height_negative_runtime_error.fpas — NEW: negative-height rejection

crates/fpas-cli/src/main_tests/
  └── standard_library.rs — MODIFY: negative-height runtime-error regression

apps/ide/README.md        — MODIFY after implementation
docs/pascal/apps/ide.md   — MODIFY after implementation
```

Do not add a new application module unless one of the listed files approaches
400 lines or the implementation introduces a genuinely separate concern.

### Implementation sequence

- [x] Add a `MessageOffset: TuiPoint` field initialized to `(0, 0)`.
- [x] Preserve the offset in every root-model reconstruction.
- [x] Add stable, unique message-scroll control and action identities.
- [x] Render `MessageText` as multiline content inside one `MakeScroll` child.
- [x] Give Messages a bounded height while leaving the editor expanding.
- [x] Accept only the matching `TuiMsg.ScrollChanged`.
- [x] Reset the offset whenever a new message replaces `MessageText`.
- [x] Keep dialogs as the active modal routing boundary.
- [x] Update current documentation after the focused tests pass.

### Required regression coverage

- [x] Initial state has a zero message offset.
- [x] Matching and unrelated `ScrollChanged` messages are distinguished.
- [x] New process results and status/error messages reset the offset.
- [x] Three diagnostic lines are visible and later lines remain clipped.
- [x] Pointer focus, Down, PageDown, and Home scroll the viewport.
- [x] Resize retains a valid surface.
- [x] Existing IDE and TUI regressions remain green.

### Acceptance criteria

- [x] Three output lines are visible on a normal terminal.
- [x] Long output scrolls through existing TUI controls.
- [x] The complete report remains stored in `MessageText`.
- [x] New output starts at its first line.
- [x] Message scrolling leaves the document unchanged.
- [x] The required TUI correction is generic and independently tested.

### Verification

Verified on 2026-07-25:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check <modified FPAS paths>
cargo run -q -p fpas-cli -- check --std-lib lib apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test --std-lib lib tests/stdlib/tui
cargo run -q -p fpas-cli -- test --std-lib lib tests/
```

## Completed slice: diagnostic source navigation

> Completed 2026-07-25.

### Outcome

Parse source locations from captured Check diagnostics, visibly select one
diagnostic in the existing Messages viewport, and let Enter move the editor
caret and viewport to that source position. Keep the complete unmodified
process report in `MessageText`.

Run output is not parsed in this slice. Project and multi-document navigation
remain out of scope.

### Diagnostic model

`IdeDiagnostic` stores only navigation data:

- `Path`: the path rendered by the compiler;
- `Line` and `Column`: one-based source coordinates;
- `MessageLine`: the zero-based line containing the diagnostic header in
  `MessageText`.

`IdeModel` owns `Diagnostics: array of IdeDiagnostic` and
`SelectedDiagnostic: option of integer`. A Check result selects its first
matching diagnostic. Every non-Check message clears both fields.

### Parsing boundary

The parser consumes normalized `MessageText`, not compiler-internal
diagnostics. A line is accepted only when it has one of these shapes:

```text
<path>:<positive-line>:<positive-column>: error[<code>]: <message>
<path>:<positive-line>:<positive-column>: warning[<code>]: <message>
Cannot build test program `<path>`: <positive-line>:<positive-column>: error[<code>]: <message>
```

Parsing searches backward from the severity marker, so Windows drive-letter
colons remain part of the path. The single-file build wrapper is removed before
path matching. Coordinates are parsed digit by digit with a bounded length;
malformed or unreasonably large values are ignored instead of raising a runtime
error. Help lines and arbitrary process output are ignored.

### File matching

Only diagnostics for the open document are retained. Compare paths after
normalizing `\` to `/` and removing leading `./` segments. Exact normalized
paths match. A relative and absolute path also match when the shorter path is a
complete slash-delimited suffix of the longer path. Do not match bare textual
suffixes.

Windows drive-letter paths are compared case-insensitively. Other paths retain
their case.

### Selection and navigation

The Messages viewport remains the existing controlled `Scroll`; no new TUI
element or routing behavior is required.

- The selected diagnostic is marked on its header line.
- A new Check report scrolls directly to its first matching diagnostic.
- Scrolling selects the last diagnostic header at or above the viewport start,
  or the first diagnostic when the viewport is above all headers.
- Enter acts only while the Messages control is focused and a diagnostic is
  selected.
- Navigation clamps stale line and column coordinates to the current document,
  focuses the editor, and places the target at the viewport origin.
- Navigation preserves text, saved baseline, path, dirty state, messages, and
  diagnostic selection.

### Intended file layout

```text
apps/ide/src/diagnostic/
  ├── model.fpas  — NEW: parsed diagnostic navigation data
  └── parser.fpas — NEW: safe header parsing and current-file matching

apps/ide/src/
  ├── app/model.fpas          — MODIFY: parsed diagnostics and selection
  ├── app/document_flow.fpas  — MODIFY: populate or clear diagnostics
  ├── app/update.fpas         — MODIFY: scroll selection and Enter navigation
  ├── app/view.fpas           — MODIFY: selected-header marker
  └── document/model.fpas     — MODIFY: clamped source-position navigation

tests/ide/
  ├── ide_diagnostic_parser_test.fpas — NEW: parsing and path matching
  ├── ide_document_navigation_test.fpas — NEW: clamped source positions
  ├── ide_update_test.fpas            — MODIFY: selection and navigation
  ├── ide_surface_test.fpas           — MODIFY: visible selection marker
  └── ide_process_integration_test.fpas — MODIFY: real Check diagnostic

apps/ide/ide-core.fpasprj — MODIFY: export diagnostic units
apps/ide/README.md        — MODIFY after implementation
docs/pascal/apps/ide.md   — MODIFY after implementation
```

### Implementation sequence

- [x] Add the diagnostic model and defensive parser.
- [x] Add safe source-position navigation to `IdeDocument`.
- [x] Populate diagnostics only from completed Check reports.
- [x] Clear stale diagnostics when any other message replaces the report.
- [x] Keep selection synchronized with the Messages viewport.
- [x] Navigate on Enter only from the focused Messages control.
- [x] Mark the selected diagnostic without changing `MessageText`.
- [x] Update current documentation after focused tests pass.

### Required regression coverage

- [x] Parse errors and warnings with relative and Windows drive-letter paths.
- [x] Ignore help lines, malformed coordinates, unrelated text, and other
  files.
- [x] Match normalized relative/absolute paths only on slash boundaries.
- [x] Select the first diagnostic and reveal its header after Check.
- [x] Update selection while scrolling through multiple diagnostics.
- [x] Enter moves the caret and viewport, focuses the editor, and preserves
  document contents and dirty state.
- [x] Stale out-of-range coordinates clamp without panic.
- [x] Enter outside Messages and messages without diagnostics do nothing.
- [x] Existing IDE and TUI regressions remain green.

### Acceptance criteria

- [x] A real failed Check produces at least one navigable diagnostic.
- [x] The selected diagnostic is visible in Messages.
- [x] Enter moves to its source line and column.
- [x] Arbitrary captured output cannot crash diagnostic parsing.
- [x] Diagnostics for another file cannot move the current document.
- [x] No compiler, VM, or `Std.Tui` API is added for this slice.

### Verification

Verified on 2026-07-25:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check apps/ide tests/ide
cargo run -q -p fpas-cli -- check --std-lib lib apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test --std-lib lib tests/stdlib/tui
cargo run -q -p fpas-cli -- test --std-lib lib tests/
```

The focused suites passed 15 IDE and 49 TUI tests. The complete FPAS suite
passed 383 tests with one expected skip.

## Next work

Continue in this order:

1. Decide whether project/workspace startup is needed before multiple open
   documents.
2. Add multi-document state only when a concrete workflow requires it.

Before implementation, extend this plan with the accepted startup target,
command-line shape, project-root behavior, Check/Run invocation, and regression
cases. Do not combine startup-target work with multiple-document state.

## Maintenance rule

Update the status and verification notes when work begins. Once the scrollable
output and any later recorded items are implemented, documented under
`docs/pascal/`, and covered by tests, remove this future plan and its index
entry.
