# Future: IDE next steps

> Status snapshot: 2026-07-26. Scrollable message output, diagnostic source
> navigation, direct `.fpasprj` ingestion, and internal `.fpasworkspace`
> ingestion are implemented. The IDE also has hierarchical menus and three
> runtime-switchable themes. Its menu, popup, status, dialog, and button chrome
> follows the Turbo Pascal 6-era Turbo Vision color hierarchy; modal dialogs
> also have shadows and mouse-draggable title bars. Project/workspace trees are
> deliberately deferred.

## Purpose

Continue the small terminal IDE in `apps/ide` without restoring the abandoned
IDE architecture and without introducing a tiled window manager. Compiler
output is usable in the single-document workflow. Project and workspace data is
retained for a later UI without expanding the current screen.

The implemented behavior is documented in:

- [`docs/pascal/apps/ide.md`](../pascal/apps/ide.md)
- [`apps/ide/README.md`](../../apps/ide/README.md)

Those files are authoritative for current behavior. Keep proposed behavior in
this document until it is implemented and tested.

## Current implementation

The IDE is a fixed, single-document `Std.Tui` application:

- it opens or starts with one UTF-8 `.fpas` source, one `.fpasprj` project, or
  one `.fpasworkspace`;
- it retains validated project metadata, original manifest text, and direct
  source paths without displaying a project tree yet;
- it retains validated workspace metadata, original manifest text, member
  order, and every direct member project without displaying workspace UI;
- it edits the source with a controlled multiline `TextArea`;
- it supports Open, Save, Check, Run, and Exit through File/Edit/Run/Options
  popup menus with mnemonics and structural shortcuts;
- it switches between Light, Dark, and Monochrome without restarting;
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
- Open, Save As, and dirty-document dialogs are modal, shadowed, movable by
  dragging their title bars, and cancelable through their `[■]` close boxes.
- dialog buttons have default/focus markers, mnemonics, block shadows,
  Enter/Space/Alt routing, and release-inside pointer activation.

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
      │   ├── open_flow.fpas    — atomic source/project/workspace Open targets
      │   ├── update.fpas       — TuiMsg routing and shortcuts
      │   └── view.fpas         — fixed IDE surface and modal dialogs
      ├── document/
      │   ├── io.fpas           — UTF-8 file operations
      │   └── model.fpas        — text, caret, viewport, path, and dirty state
      ├── diagnostic/
      │   ├── model.fpas        — parsed source navigation data
      │   └── parser.fpas       — captured diagnostic header parsing
      ├── project/
      │   ├── loader.fpas       — project file I/O entry point
      │   ├── manifest.fpas     — typed TOML field validation
      │   ├── model.fpas        — retained project session data
      │   ├── session.fpas      — atomic project and initial-document loading
      │   └── sources.fpas      — main/include/exclude path resolution
      ├── workspace/
      │   ├── loader.fpas       — workspace file and member loading
      │   ├── manifest.fpas     — typed workspace TOML validation
      │   ├── model.fpas        — retained workspace and member projects
      │   └── session.fpas      — atomic workspace and initial-document loading
      ├── theme/
      │   ├── model.fpas        — bundled theme identities
      │   └── palette.fpas      — complete Tui palettes
      └── process/
          └── runner.fpas       — Check/Run invocation and output formatting
```

The regression suite is in `tests/ide/ide-tests.fpasprj`. It covers the model,
surface, document I/O, dirty-document decisions, dialogs, pointer interaction,
shortcuts, diagnostic parsing and navigation, process argument/format behavior,
process integration, and the document lifecycle. The current 25-test suite also
covers hierarchical menu hover through a real IDE surface, every bundled theme
role, recovery and continued input after a burst of terminal resizes, the
first-line Enter regression, wide-glyph diagnostic navigation, case-insensitive
Open target extensions, and defensive project/workspace manifest failures.

## Current coverage audit

> Audited 2026-07-26.

The audit compared the implemented source, current user documentation, this
plan, manifests, and existing tests. It added focused regressions instead of
duplicating generic `Std.Tui` coverage:

- `ide_menu_pointer_test.fpas` drives closed-menu hover, root switching, nested
  submenu hover, command selection without activation, movement outside, and
  the final theme click;
- `ide_editor_resilience_test.fpas` keeps the first line visible after Enter,
  queues repeated valid and terminal-too-small resizes, then proves that editor
  input and the final surface still advance;
- `ide_dialog_surface_test.fpas` renders every IDE dialog and drives a real
  title-bar press, drag, repaint, and release; generic Tui button regressions
  cover focus/default markers, mnemonics, disabled state, keyboard activation,
  press capture, drag-out cancellation, and release-inside activation;
- `dialog_pointer_test.fpas` covers centered movable-dialog layout, title and
  shadow roles, model-owned drag state, movement, release, and edge clamping;
- `ide_open_target_case_test.fpas` opens uppercase `.FPAS`, `.FPASPRJ`, and
  `.FPASWORKSPACE` targets, including an uppercase workspace member;
- project/workspace loader tests now exercise missing files, wrong tables and
  scalar types, missing and empty required fields, mixed arrays, invalid main
  paths, complete exclusion, wrong extensions, and duplicate members/names;
- document and theme tests cover empty and wide-glyph navigation plus every
  foreground/background role of all three bundled palettes.

Verified on 2026-07-26:

```text
fpas fmt --check apps/ide tests/ide
fpas check --std-lib lib apps/ide/ide.fpasworkspace
fpas test --std-lib lib --strict --jobs 4 tests/ide/ide-tests.fpasprj
fpas test --std-lib lib tests/stdlib/tui
fpas test --std-lib lib tests/suite.fpasprj
cargo fmt --all -- --check
cargo build --workspace
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace
```

The focused IDE suite passed 25 tests, the TUI suite passed 52 tests, and the
complete FPAS suite passed 396 tests with one expected skip. The Rust workspace
format, build, strict Clippy gate, and test suite passed.

## Known limits

Deliberate limits are:

- one open document only;
- no project tree or dependency-project view;
- no workspace member selector;
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

### Implemented file layout

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

There is no automatically selected next IDE slice. Choose the next concrete
single-document workflow before implementation. Project/workspace UI remains
explicitly deferred:

1. Add a project/workspace tree only after a separate user request.
2. Add workspace member selection together with that requested workflow.
3. Add multi-document state only when a concrete workflow requires it.

Do not add a tree, selector, dependency view, or multiple-document state merely
because the data is now available.

## Completed intermediate slice: direct project ingestion

> Completed 2026-07-25.

### Outcome

Opening a `.fpasprj` establishes a project session. The IDE reads and parses the
manifest itself, resolves its direct source files, retains that information in
the application model, and opens one initial source in the existing
single-document editor.

This slice supplies the data boundary for a later Visual Studio-style project
tree. It does not render that tree.

### Project model

`IdeProject` retains:

- manifest `Path`, project `Root`, and the complete original `ManifestText`;
- validated project `Name` and `Kind`;
- resolved `MainFile` for program projects;
- stable, deduplicated `SourceFiles` for the direct project.

The project kind is a typed enum for `program`, `library`, and `test`.
Dependency-project sources are not flattened into `SourceFiles`; a later tree
can represent dependencies as separate projects.

### Manifest and source resolution

The IDE uses `Std.Fs.ReadText`, `Std.Toml.Parse`, `Std.Path`, and `Std.Fs.Glob`
instead of compiler-internal project APIs.

- `[project].name` and `[project].kind` are required.
- `program` requires `[project].main`; `library` and `test` reject it.
- `[sources].include` is a required non-empty string array.
- `[sources].exclude` is an optional string array.
- Patterns are resolved relative to the manifest directory.
- Every include pattern must match and every retained file must end in
  `.fpas`; exclude patterns may match nothing.
- Duplicate and excluded paths are removed deterministically.
- A program main file is retained exactly once even when an include also
  matches it.
- TOML, type, glob, missing-file, and extension errors return an IDE message
  without replacing the current project or document.

The original manifest text is retained so later project features do not need
to reread or reconstruct sections that this slice does not interpret.

### Session behavior

- Startup and Open accept `.fpas` and `.fpasprj`, case-insensitively.
- Opening a standalone `.fpas` clears the project session.
- Opening a program project loads its main file.
- Opening a library or test project loads its first resolved source.
- Project open is atomic: manifest and initial source must both load before
  replacing the current session.
- Dirty-document Save/Discard/Cancel protection remains unchanged.
- The fixed path row shows the project name together with the active document.
- Save continues to write only the active source document.
- Check and Run save the active source first, then invoke `fpas` with the
  retained project manifest path. Standalone documents keep their current
  command behavior.
- Diagnostic navigation remains limited to the active source document.

### Implemented file layout

```text
apps/ide/src/project/
  ├── loader.fpas   — NEW: project file I/O entry point
  ├── manifest.fpas — NEW: typed TOML field validation
  ├── model.fpas    — NEW: retained direct-project session data
  ├── session.fpas  — NEW: atomic project and initial-document loading
  └── sources.fpas  — NEW: main/include/exclude path resolution

apps/ide/src/
  ├── app/model.fpas          — MODIFY: optional project session
  ├── app/document_flow.fpas  — MODIFY: atomic project open and command target
  └── app/view.fpas           — MODIFY: project-aware path row

tests/ide/
  ├── ide_project_loader_test.fpas — NEW: manifest and glob resolution
  ├── ide_project_open_test.fpas   — NEW: startup/Open session behavior
  └── ide_project_process_test.fpas — NEW: Check and Run target the project

apps/ide/ide-core.fpasprj — MODIFY: export project units
apps/ide/README.md        — MODIFY after implementation
docs/pascal/apps/ide.md   — MODIFY after implementation
```

### Implementation sequence

- [x] Add the retained project model.
- [x] Parse and validate required manifest fields without panics.
- [x] Resolve include/exclude patterns relative to the project root.
- [x] Load the initial document atomically with the project.
- [x] Accept project paths at startup and through Open.
- [x] Preserve or clear project state through every model reconstruction.
- [x] Target Check and Run at the retained manifest.
- [x] Display project identity without adding the project tree.
- [x] Update current documentation after focused tests pass.

### Required regression coverage

- [x] Load program, library, and test project metadata.
- [x] Retain the original manifest text and normalized direct source list.
- [x] Resolve includes, excludes, duplicates, and main-file overlap.
- [x] Reject malformed TOML, missing/wrong fields, invalid kinds, unmatched
  includes, non-FPAS matches, and missing program main.
- [x] Startup and Open establish a project session and initial document.
- [x] Failed project open preserves the current document and project.
- [x] Opening a standalone source clears a previous project.
- [x] Dirty-document protection also guards project Open.
- [x] Check uses the project manifest while saving the active source.
- [x] Existing source-only, diagnostic, IDE, and TUI regressions remain green.

### Acceptance criteria

- [x] `IdeModel` locally retains the open project and all direct source paths.
- [x] No compiler-internal project loader is required by the IDE.
- [x] A later project tree can be built without rereading the manifest.
- [x] Project opening is atomic and does not lose dirty work.
- [x] `.fpasworkspace`, dependency trees, tree UI, and multiple documents are
  not introduced.

### Verification

Verified on 2026-07-25:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check apps/ide tests/ide
cargo run -q -p fpas-cli -- check apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test tests/stdlib/tui
cargo run -q -p fpas-cli -- test tests/
```

The focused suites passed 18 IDE and 49 TUI tests. The complete FPAS suite
passed 386 tests with one expected skip. The Rust workspace build and test
suite passed.

The strict workspace Clippy gate also passes as of 2026-07-26.

## Completed intermediate slice: internal workspace ingestion

> Completed 2026-07-25.

### Outcome

Opening a `.fpasworkspace` establishes a retained workspace session without
adding workspace UI. The IDE parses the manifest itself, resolves and validates
every member `.fpasprj`, and stores the complete member project models in
declared order. The first member becomes the active project and supplies the
initial source document.

### Implemented ownership

```text
apps/ide/src/workspace/
  ├── loader.fpas   — workspace file I/O, member paths, and project loading
  ├── manifest.fpas — typed [workspace] name/member validation
  ├── model.fpas    — retained manifest data and ordered member projects
  └── session.fpas  — atomic workspace, active project, and document loading
```

`IdeModel.Workspace` retains the workspace independently from
`IdeModel.Project`, which remains the active member used by the existing
single-document and process flows.

### Implemented behavior

- [x] Startup and Open accept `.fpasworkspace` case-insensitively.
- [x] Retain normalized path/root, original manifest text, name, member order,
  and complete direct project models.
- [x] Reject malformed manifests, empty members, non-project members,
  duplicate paths, duplicate case-insensitive project names, and member load
  failures.
- [x] Replace workspace, active project, and document only after the complete
  initial session loads successfully.
- [x] Opening a source clears workspace and project state.
- [x] Opening a project directly clears workspace state.
- [x] Preserve dirty-document Save/Discard/Cancel protection.
- [x] Check and Run continue to target the active member project manifest.
- [x] Add no workspace tree, selector, dependency view, or multi-document UI.

### Regression coverage

```text
tests/ide/
  ├── ide_workspace_loader_test.fpas  — manifest/member validation and ordering
  ├── ide_workspace_open_test.fpas    — startup, Open, atomicity, and clearing
  └── ide_workspace_process_test.fpas — active-member Check and Run
```

### Verification

Verified on 2026-07-25:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
fpas fmt --check apps/ide tests/ide
fpas check apps/ide/ide.fpasworkspace
fpas test tests/ide/ide-tests.fpasprj
fpas test tests/
```

The focused IDE suite passed 21 tests. The complete FPAS suite passed 389 tests
with one expected skip. The Rust workspace build and test suite passed. One
Windows sidecar lock test failed once with `Access denied`; its isolated rerun
and the repeated complete workspace test both passed.

## Maintenance rule

Update the status and verification notes when work begins. Once the scrollable
output and any later recorded items are implemented, documented under
`docs/pascal/`, and covered by tests, remove this future plan and its index
entry.
