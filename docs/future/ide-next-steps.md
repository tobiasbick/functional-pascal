# Future: IDE next steps

> Status snapshot: 2026-07-25. The current single-document IDE is implemented
> and supported. This document records only the work that should follow.

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

The completed baseline was verified with:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace
cargo run -q -p fpas-cli -- fmt --check apps/ide tests/ide
cargo run -q -p fpas-cli -- check --std-lib lib apps/ide/ide.fpasworkspace
cargo run -q -p fpas-cli -- test --std-lib lib tests/ide/ide-tests.fpasprj
cargo run -q -p fpas-cli -- test --std-lib lib tests/
```

## Known limits

The first practical limitation is the Messages window. It has a fixed
one-line viewport, so all later lines of compiler diagnostics and program
output are clipped.

Other deliberate limits are:

- no diagnostic parsing or jump-to-source action;
- one open document only;
- no `.fpasprj` or `.fpasworkspace` startup target;
- synchronous Check and Run;
- no stdin connection for child programs;
- no help system;
- no tiled windows or tiled window manager.

Do not address these other limits in the next slice.

## Next slice: scrollable message output

### Outcome

Replace the one-line Messages window with a bounded multiline viewport. The
latest complete Check/Run report remains in the model and can be scrolled
vertically without changing the editor document.

### Reuse boundary

Use the existing controlled `TuiElement.Scroll` API:

```text
MakeScroll(Id, Offset, ChangeAction, Child)
TuiMsg.ScrollChanged(Source, Action, Offset)
```

`Std.Tui` already routes scroll keyboard input and pointer focus. Do not create
an IDE-specific scrolling implementation. Change `Std.Tui` only if a focused
test first demonstrates that its layout cannot express a bounded multiline
viewport; keep any such correction generic and independently tested.

### Intended file layout

```text
apps/ide/src/app/
  ├── actions.fpas       — MODIFY: message-scroll control and action identities
  ├── model.fpas         — MODIFY: controlled message offset
  ├── update.fpas        — MODIFY: accept the matching ScrollChanged message
  └── view.fpas          — MODIFY: bounded Messages window containing Scroll

tests/ide/
  ├── ide_model_test.fpas       — MODIFY: initial message offset
  ├── ide_update_test.fpas      — MODIFY: valid and unrelated scroll messages
  ├── ide_surface_test.fpas     — MODIFY: multiline viewport and clipping
  └── ide_interaction_test.fpas — MODIFY: focus, keyboard scrolling, and resize

apps/ide/README.md        — MODIFY after implementation
docs/pascal/apps/ide.md   — MODIFY after implementation
```

Do not add a new application module unless one of the listed files approaches
400 lines or the implementation introduces a genuinely separate concern.

### Implementation sequence

1. Add a `MessageOffset: TuiPoint` field to `IdeModel`, initialized to `(0, 0)`.
2. Preserve the offset in every root-model reconstruction.
3. Add stable, unique message-scroll control and change-action identities using
   the next unused values.
4. Render `MessageText` as multiline content inside one `MakeScroll` child.
5. Give the Messages window a predictable bounded height while leaving the
   editor as the expanding region.
6. Accept `TuiMsg.ScrollChanged` only when both the message control and action
   identities match; unrelated scroll messages must leave the model unchanged.
7. Reset `MessageOffset` to `(0, 0)` whenever a new status or process report
   replaces `MessageText`, so the beginning of new output is visible.
8. Preserve the current modal boundary: active dialogs retain focus and block
   application shortcuts and message scrolling.
9. Update current documentation only after the behavior passes its tests.

### Required regression coverage

- Initial state has a zero message offset.
- A matching `ScrollChanged` updates only `MessageOffset`.
- An unrelated source or action is ignored.
- A new Check/Run result and an error message reset the offset.
- Multiple diagnostic lines are visible within the viewport.
- Output beyond the viewport is clipped rather than painted over the editor or
  status line.
- Pointer focus followed by Down/PageDown scrolls the message viewport.
- Home returns to the first output line.
- Resize keeps the surface valid and clamps effective scrolling through the TUI
  routing behavior.
- Existing editor, dialog, dirty-document, shortcut, and process tests remain
  green.

### Acceptance criteria

- At least several lines of output are visible on a normal terminal.
- Long output is vertically scrollable through existing TUI controls.
- The complete report remains stored in `MessageText`.
- New output starts at its first line.
- Editor text, caret, viewport, dirty state, and document path are unaffected by
  message scrolling.
- No new compiler, VM, or `Std.Tui` behavior is introduced unless required by a
  separately demonstrated TUI defect.
- All verification commands from the baseline pass.

## Work after the next slice

After the scrollable Messages window is complete, continue in this order:

1. Parse compiler diagnostic locations from Check output and maintain a
   selected diagnostic.
2. Let Enter on a selected diagnostic move the editor caret and viewport to its
   line and column.
3. Decide whether project/workspace startup is needed before multiple open
   documents.
4. Add multi-document state only when a concrete workflow requires it.

Each item should receive its own focused plan or extension of this file before
implementation. Do not combine these items with the scrollable-output slice.

## Maintenance rule

Update the status and verification notes when work begins. Once the scrollable
output and any later recorded items are implemented, documented under
`docs/pascal/`, and covered by tests, remove this future plan and its index
entry.
