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
visibility, enabled state, size hints, size policies, default restoration, and application-close
cleanup, plus direct container attachment and destructive removal.
Application coverage verifies FIFO callback posting, including callbacks posted during a drain;
bounded iteration budgets; automatic single start; orderly quit and stop; pre-tick posted quit;
retained application size; desktop resize propagation; and automatic dirty desktop layout passes.
Custom-view lifecycle coverage verifies visible parent state during attach, structural cleanup before
detach, live senders during detach, post-layout resize ordering, resize coalescing, deferred changes
from inside a resize callback, self-destruction, continued sibling delivery, and application-close
detachment.
Measurement-value coverage verifies bounded and unbounded constraints plus ordered measurement results.
Size-policy coverage verifies independent axes and every uniform policy constructor.
Layout-value coverage verifies margin construction, axis totals, and mixed or uniform alignment.
Layout-item coverage verifies fixed and expanding spacers plus view, nested-layout, and spacer item
descriptions with alignment and stretch. Live-list coverage verifies stable order, removal, clearing,
duplicate rejection, cycle rejection, exclusive container-or-layout ownership, child detachment, and
nested destruction while preserving view-tree ownership.
Container coverage also verifies recursive destruction of nested child subtrees. Desktop coverage
verifies its single root identity, direct child ownership, and invalidation on application close.
`TuiLayout` coverage verifies the same slot reuse, stale-generation, tag, and application-close
contracts. Container-layout coverage verifies attachment, direct layout destruction, replacement,
and destruction with the container. Container-pass coverage verifies initial arrangement, idle-pass
suppression, resize detection, local coordinates, view-input invalidation, nested propagation, and
coalescing. Layout-engine coverage verifies typed directions, live margins and spacing, hidden-view
exclusion, constrained recursive measurement, policy evaluation, stable remainder allocation,
finite maximum sizes, alignment, and nested rectangle assignment.
Terminal-too-small coverage verifies empty-container fit, two-axis shortages, preserved minimum
geometry, desktop forwarding, and recovery after resize.
Scroll-view coverage verifies empty viewport metrics, preferred content extent, per-axis maximum and
clamped offsets, negative local allocation, content shrinkage, viewport growth, invalidation, and
layout ownership cleanup. It also verifies recovery after a pass through the exposed container
identity. A runtime-error canary verifies that callers cannot assign negative offsets.
Grid coverage additionally verifies placement overlap rejection, positive spans, inferred tracks,
span measurement, stable two-axis growth, per-axis alignment, direct child cleanup, and nesting
through the common layout dispatcher. Form coverage verifies paired insertion, two-column
measurement and allocation, typed row lookup, non-destructive row removal, and cleanup after direct
view destruction. Stacked coverage verifies stable maximum-page measurement, hidden-page exclusion,
active-page allocation, current-index changes, removal clamping, and empty-stack normalization.

Interactive smoke tests remain useful for terminal compatibility, but they are not the primary regression mechanism.
