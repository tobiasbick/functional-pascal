# TUI Plan: Widget Polish

**Status:** next active plan.

Goal: improve retained widget ergonomics without changing the retained engine contract.

## Scope

- Small user-visible polish for existing controls.
- Better examples for shipped widgets.
- Focused docs updates under `docs/pascal/std/tui/` only when behavior already exists or is added in
  the same change.

## Work Items

- [ ] Review button default/cancel behavior in framed dialogs and decide whether a public helper is
  warranted or existing command binding is enough.
- [ ] Add an example that combines frame layout, labels, input line, checkbox/radio, list box, and
  memo in one small settings dialog.
- [ ] Improve list box and memo empty-state rendering if current behavior is hard to inspect in
  headless screenshots.
- [ ] Audit control focus visuals for consistency across active frame, inactive frame, and modal
  dialog palettes.
- [ ] Add any missing FPAS workflow tests for the example-level control combinations above.

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
