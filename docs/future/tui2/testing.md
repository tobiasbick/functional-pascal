# Std.Tui2 testing contract

Std.Tui2 is designed for deterministic headless verification from the first implementation phase.

## Headless application

`TuiApplication.OpenForTest(Size)` creates the same FPAS registry, layout, routing, lifecycle, and rendering state as an interactive application without changing terminal modes.

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

Dedicated tests cover cross-application handles, stale generations, worker-task UI calls, forbidden paint mutation, callback panic cleanup, and terminal-open rollback.

Interactive smoke tests remain useful for terminal compatibility, but they are not the primary regression mechanism.
