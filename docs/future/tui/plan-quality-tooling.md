# TUI Plan: Quality, Tooling, and Maintenance

**Status:** next active plan.

Goal: make retained TUI regressions easier to catch before implementation work reaches user-facing
docs or examples.

## Scope

- Headless integration coverage for event ordering and redraw behavior.
- Property-style coverage for resize/key/mouse bursts where deterministic examples are brittle.
- Real-terminal smoke checklist alignment.
- Link hygiene for Rust doc comments and `docs/pascal/` references.

## Work Items

- [ ] Add a focused host-event ordering test for resize bursts followed by key and mouse input.
- [ ] Add a retained redraw regression that verifies dirty-region coalescing after overlapping
  frame move, close, and menu overlay repaint.
- [ ] Add property-style tests for scroll/thumb geometry edge cases: empty content, viewport equal
  content, huge content, and repeated drag deltas.
- [ ] Add a doc-link audit for TUI Rust comments that reference `docs/future/tui/` or
  `docs/pascal/std/tui/`.
- [ ] Update [`docs/pascal/std/tui/terminal-checklist.md`](../../pascal/std/tui/terminal-checklist.md)
  if real-terminal smoke expectations drift from headless behavior.
- [ ] Add a short contributor note to [`docs/pascal/std/tui/app/testing.md`](../../pascal/std/tui/app/testing.md)
  explaining which TUI test directory to use for host, scene, controls, menu, modals, and frames.

## Acceptance Criteria

- New tests fail on at least one realistic regression class: event ordering, dirty repaint, scroll
  geometry, or stale docs links.
- The plan does not add new public `Std.Tui` behavior.
- Any changed current docs under `docs/pascal/` describe shipped behavior only.

## Verification

Run after implementation:

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/
```

If only markdown under `docs/future/tui/` changes, `git diff --check` is sufficient.
