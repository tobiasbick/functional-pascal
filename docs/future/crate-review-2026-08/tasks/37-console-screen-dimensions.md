# Task 37 — Keep reported console dimensions and retained state synchronized

Status: complete
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`ScreenWidth`/`ScreenHeight` report the dimensions of the same retained console state used by
coordinates, windows, cells, frames, and rendering.

## Contract

[`Screen utilities`](../../../pascal/std/console/screen-misc.md) defines these functions as current
console dimensions; retained-screen APIs use the same character-cell coordinate space.

## Verified cause

`console/operations/window.rs` queries live `crossterm::terminal::size()` in immutable getters and
returns it without calling `ConsoleState::resize`. The caller can receive a new width/height while
the back buffer, active window, and coordinate checks still use old dimensions.

## Fix

Synchronize terminal size before returning both getters, using the existing `sync_terminal_size`
path so resize/clamping behavior has one authority. Adjust mutability at the intrinsic dispatch
boundary as needed; do not duplicate resize logic in the getters.

If live terminal size cannot be queried, return retained state dimensions as today.

## Tests

- Use the existing console test double or inject the existing terminal-size seam; do not add a
  public host API solely for tests.
- A simulated resize changes reported dimensions and retained screen/window dimensions together.
- Failed/unavailable size query leaves prior state intact.

## Verify

```text
cargo test -p fpas-std
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Getters cannot disagree with retained state after a successful size query.
- Resize invariants and existing headless tests pass.
- Docs unchanged unless the fallback behavior needs an explicit contract.

## Progress

- Base commit: 74b16b7b
- Current step: verify getter-driven terminal resize synchronization
- Files changed: console state/query operations and screen tests
- Verification: full workspace definition of done passed on 2026-08-19
- Blockers: none
