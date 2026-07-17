# Std.Tui2 testing contract

Std.Tui2 is designed for deterministic headless verification from the first implementation phase.

## Headless application

The implemented headless application registry is documented in the current
[`Std.Tui2` reference](../../pascal/std/tui2/README.md). Its future layout, routing, and rendering
state must remain identical to the interactive application without changing terminal modes.

Tests inject `Std.Console.Event` values into the application's event source and advance the loop one iteration at a time.

## Determinism

- Headless and live rendering use the same display-width and clipping rules.
- Cell remainder allocation follows stable item order.
- Injected events are FIFO.
- Posted main-task callbacks are FIFO.
- Tests use a virtual monotonic clock for ticks, repeat behavior, and timeouts.
- No test depends on wall-clock sleeps or an interactive terminal.

## Assertion levels

| Level | Examples |
| --- | --- |
| Pure values | Geometry, size policies, palette resolution, command identity. |
| Registry | Generations, ownership, stale handles, destruction order. |
| Layout | Minimum/preferred allocation, nesting, clipping, resize. |
| Routing | Focus, capture, modal isolation, actions, unhandled input. |
| Lifecycle | Attach, measure, resize, paint, focus, close order. |
| Screen | Exact cells, Unicode width, colors, wide-glyph repair. |
| End to end | Small FPAS applications driven entirely by injected events. |

Every public control requires at least one keyboard test, one mouse test when applicable, one layout-resize test, and one screen-output test.

## Failure canaries

Remaining failure canaries cover worker-task UI calls, forbidden paint mutation, callback panic
cleanup, and terminal-open rollback.

Implemented registry coverage includes `TuiView` slot reuse, stale generations, tags, bounds,
visibility, enabled state, and application-close cleanup, plus direct container attachment and
destructive removal.
Container coverage also verifies recursive destruction of nested child subtrees. Desktop coverage
verifies its single root identity, direct child ownership, and invalidation on application close.
`TuiLayout` coverage verifies the same slot reuse, stale-generation, tag, and application-close
contracts before layout items and allocation are introduced. Container-layout coverage verifies
attachment, direct layout destruction, replacement, and destruction with the container.

Interactive smoke tests remain useful for terminal compatibility, but they are not the primary regression mechanism.
