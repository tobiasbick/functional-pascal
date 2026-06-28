# TUI Plan: Widget Polish

**Status:** complete.

Goal: improve retained widget ergonomics without changing the retained engine contract.

## Scope

- Small user-visible polish for existing controls.
- Better examples for shipped widgets.
- Focused docs updates under `docs/pascal/std/tui/` only when behavior already exists or is added in
  the same change.

## Work Items

- [x] Review button default/cancel behavior in framed dialogs and decide whether a public helper is
  warranted or existing command binding is enough.

  Decision: no new public helper for now. OK/Cancel buttons use command ids, and Enter/Escape use
  `HostBindCommandToActiveModal` after the dialog opens so button clicks and keyboard defaults share
  the same `OnCommand` path.
- [x] Add an example that combines frame layout, labels, input line, checkbox/radio, list box, and
  memo in one small settings dialog.

  Added `examples/pascal/tui/settings_dialog.fpas`.
- [x] Improve list box and memo empty-state rendering if current behavior is hard to inspect in
  headless screenshots.

  Empty list boxes and unfocused empty memos now paint a disabled `(empty)` placeholder.
- [x] Audit control focus visuals for consistency across active frame, inactive frame, and modal
  dialog palettes.
  Audit result: focused controls share the dialog active palette; input-line and memo focus is shown
  by the cursor cell using that same palette. Window frames intentionally distinguish active and
  inactive chrome, while dialog frames keep gray chrome in both states.
- [x] Add any missing FPAS workflow tests for the example-level control combinations above.
  Added `tests/tui/modals/tui_settings_dialog_workflow_test.fpas` for the settings-dialog control
  mix and modal close flow.

## Acceptance Criteria

- No new compatibility layer or duplicate widget model.
- Examples remain runnable demos, not `*_test.fpas` files.
- Tests cover any changed behavior through `tests/tui/controls/`, `tests/tui/modals/`, or
  `tests/tui/frames/` as appropriate.

## Verification

Run after implementation:

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/
```

After editing `.fpas` examples or tests, run the FPAS formatter/check required by the project
workflow.
