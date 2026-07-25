# Future: IDE next steps

> Status snapshot: 2026-07-25. Scrollable message output is implemented. The
> next work is diagnostic parsing and source navigation.

## Purpose

Continue the small terminal IDE in `apps/ide` without restoring the abandoned
IDE architecture and without introducing a tiled window manager. The next
increment should make compiler and program output usable before expanding the
document or project model.

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
      └── process/
          └── runner.fpas       — Check/Run invocation and output formatting
```

The regression suite is in `tests/ide/ide-tests.fpasprj`. It covers the model,
surface, document I/O, dirty-document decisions, dialogs, pointer interaction,
shortcuts, process argument/format behavior, process integration, and the
document lifecycle.

## Known limits

Deliberate limits are:

- no diagnostic parsing or jump-to-source action;
- one open document only;
- no `.fpasprj` or `.fpasworkspace` startup target;
- synchronous Check and Run;
- no stdin connection for child programs;
- no help system;
- no tiled windows or tiled window manager.

Do not address these other limits in the next slice.

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
  └── fixed_height_negative_runtime_error.fpas— NEW: negative-height rejection

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
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check <modified FPAS paths>
cargo run -q -p fpas-cli -- check --std-lib lib apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test --std-lib lib tests/stdlib/tui
cargo run -q -p fpas-cli -- test --std-lib lib tests/
```

The workspace Clippy command is currently blocked by the unrelated existing
`clippy::derived_hash_with_manual_eq` finding for `SharedStr` in
`crates/fpas-bytecode/src/value/mod.rs`. This IDE slice does not change that
file.

## Next work

Continue in this order:

1. Parse compiler diagnostic locations from Check output and maintain a
   selected diagnostic.
2. Let Enter on a selected diagnostic move the editor caret and viewport to its
   line and column.
3. Decide whether project/workspace startup is needed before multiple open
   documents.
4. Add multi-document state only when a concrete workflow requires it.

Before implementing diagnostic parsing, extend this plan with the accepted
diagnostic model, parsing boundary, file-matching rules, and regression cases.
Do not start project or multi-document work in that slice.

## Maintenance rule

Update the status and verification notes when work begins. Once the scrollable
output and any later recorded items are implemented, documented under
`docs/pascal/`, and covered by tests, remove this future plan and its index
entry.
