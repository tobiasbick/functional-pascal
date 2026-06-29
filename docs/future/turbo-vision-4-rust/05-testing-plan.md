# Testing Plan

The rewrite must replace the old retained-view tests with tests that assert user-visible Turbo Vision behavior.

## Principles

- Prefer FPAS regression tests for public `Std.Tui` behavior.
- Use Rust VM tests for bridge invariants that FPAS cannot express.
- Do not preserve tests for deleted internals.
- Keep headless tests deterministic.
- Do not require a human terminal for CI-style local verification.

## Minimum Headless Capabilities

The spike must prove at least one of these paths:

- use upstream `turbo-vision` test utilities;
- use a custom backend through Turbo Vision's terminal backend abstraction;
- use screen buffer access such as `Terminal::buffer`;
- inject events through Turbo Vision's event queue or an adapter owned by `fpas-vm`.

Required operations:

- [ ] create test application with fixed width and height
- [ ] inject key event
- [ ] inject mouse event or command event
- [ ] pump one event turn
- [ ] query command callback result
- [ ] query screen line or screen cell
- [ ] close without leaving terminal raw mode active

## Test Categories

### Rust Tests

Use Rust tests for:

- handle table validity
- invalid handle diagnostics
- callback re-entry rules
- modal result propagation
- terminal cleanup on error
- ownership and drop behavior

### FPAS Tests

Use FPAS tests for:

- opening and closing an application
- creating a window
- adding a button
- dispatching a button command
- dialog OK/Cancel result
- input line text state
- menu command dispatch
- status line display, if exposed

### Manual Terminal Checks

Keep a short manual checklist after automated tests exist:

- real terminal starts in alternate screen
- mouse works for buttons and menus
- window dragging works
- resize handling works
- terminal state restores after normal exit
- terminal state restores after runtime error

## Old Tests to Delete or Rewrite

Delete tests whose only purpose was validating the old engine:

- retained view tree shape
- frame-specific inner viewport clipping
- old menu overlay compositor
- `HostProcessNext` integer process tags
- `QuerySceneGraph` snapshots
- old `ViewId` state query records

Rewrite tests that still express user-visible behavior:

- button command dispatch
- modal dialogs
- menu activation
- input controls
- screen rendering
- focus traversal

## Verification Commands

Baseline for implementation phases:

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -p fpas-cli -- test tests/
```

After FPAS source edits:

```text
fpas fmt --check tests/ examples/ apps/
```

Use narrower targeted commands while developing, but finish broad.
