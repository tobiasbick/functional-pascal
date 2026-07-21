# Std.Tui3 testing contract

Std.Tui3 is designed for deterministic headless verification from the first implementation
phase.

## Headless host

Tests create a host with `OpenForTest`, supply `Update` and `View`, inject `TuiMsg`
values, and advance with `RunIterations` or single-step injection. Terminal modes stay
unchanged.

Headless and interactive paths share layout, paint, and message rules.

## Determinism

- Same display-width and clipping rules headless and live.
- Stable leftover-cell allocation order.
- Injected messages are FIFO.
- Posted callbacks are FIFO.
- Tests use a virtual monotonic clock for ticks and timeouts.
- No test depends on wall-clock sleeps or an interactive terminal.

## Assertion levels

| Level | Examples |
| --- | --- |
| Pure values | Geometry, policies, palette resolution, action ids. |
| Update | Given model + message → expected model and command. |
| View | Given model → expected element shape (title, actions, conditionals). |
| Layout | Minimum/preferred allocation, nesting, clipping, resize. |
| Routing | Focus target, modal subtree, action message from key/pointer. |
| Screen | Exact cells, Unicode width, colors, wide-glyph repair. |
| End to end | Small FPAS programs driven only by injected messages. |

Prefer testing `Update` and `View` as pure functions before full host loops.

Every public interactive element needs at least one keyboard test, one pointer test when
applicable, one layout/resize test, and one screen-output test.

## Failure canaries

- Worker-task host calls without posting.
- Panic inside `Update` / `View` / paint leaves a clear diagnostic.
- Terminal-open rollback on failure.
- Invalid action id construction.
- Quit discards pending posts that have not started.

## What not to test as public API

- Generational handle reuse (Tui2 concept — absent).
- Widget event-property assignment order.
- `OpenModal` object state.

## Suite placement

FPAS regression tests live under `tests/stdlib/tui3/` once code exists, bundled through
[`tests/suite.fpasprj`](../../../tests/suite.fpasprj). Examples, if any, go under
`examples/` and are not named `*_test.fpas`.
