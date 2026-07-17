# Std.Tui2 implementation phases

## Phase 0 — Manifest-backed source standard library

- **Complete.** `lib/stdlib.fpasprj` is the authoritative source standard-library manifest.
- **Complete.** Public exports, private multi-segment `Std.*` implementation units, and manifest-trusted namespace validation are enforced.
- **Complete.** `run`, `check`, and `test` load the selected manifest; `--std-lib` replaces the complete library.
- **Complete.** Regression coverage verifies default discovery, override, private-unit rejection, intrinsic collisions, and namespace reservation.

Completion contract: [source-library.md](source-library.md).

## Phase 1 — Geometry, text, cells, and canvas

- **Complete prerequisite.** Static record functions are implemented with no overloads and type-only call resolution.
- **Complete.** Geometry is split into private `Geometry/Point.fpas`, `Geometry/Size.fpas`, and `Geometry/Rect.fpas` units behind the public `Std.Tui2` facade.
- **Complete.** `TuiPoint.Create`, `TuiSize.Create`, `TuiRect.Create`, and the distinct `TuiRect.From...` conversions replace free geometry factories.
- **Complete.** `Std.Console.DisplayWidth` measures extended grapheme clusters with deterministic one- and two-column terminal widths.
- Extend console cells to accept one grapheme cluster and preserve wide-glyph continuation invariants.
- **Complete.** Tui2 has source-level `TuiColor`, `TuiStyleRole`, `TuiStyle`, and `TuiCell` values.
- **Complete.** `TuiPalette` maps every `TuiStyleRole` to an immutable `TuiStyle` value.
- Implement clipping and the headless cell surface.
- Implement the transient `TuiCanvas` drawing boundary.

Completion contracts: [geometry.md](geometry.md), [text-and-cells.md](text-and-cells.md), and the pure-value section of [testing.md](testing.md).

## Phase 2 — Application registry and runtime safety

- Implement application-scoped generational handles and explicit typed conversions.
- Implement the desktop root, parent-child ownership, destruction, stale-handle diagnostics, and tags.
- Add `OpenForTest` and transactional interactive terminal acquisition.
- Add the VM terminal-mode restoration safety net.
- Add the generic typed FIFO main-task post queue and enforce main-task-only UI mutation.

Completion contracts: [handles-and-ownership.md](handles-and-ownership.md), [runtime-boundary.md](runtime-boundary.md), and the registry/failure sections of [testing.md](testing.md).

## Phase 3 — Layout engine

- Implement `TuiMeasureSpec` and `TuiMeasureResult`, including width-dependent measurement.
- Implement minimum, preferred, and maximum size calculation.
- Implement per-axis policies, stretch factors, spacers, margins, spacing, and alignment.
- Implement horizontal and vertical layouts, followed by grid, form, and stacked layouts.
- Support nesting, deterministic cell remainders, clipping below nested minimums, and the terminal-too-small overlay.
- Add `TuiScrollView` for explicitly reachable overflow.

Completion contract: [layout.md](layout.md) and the layout section of [testing.md](testing.md).

## Phase 4 — Lifecycle, event routing, and actions

Language gate: capturing closures, bound record methods, record properties, and event properties
must be complete before this phase starts. See the ordered sequence in
[the future roadmap](../README.md#recommended-tui2-language-sequence).

- Implement the bounded application loop, posted callback draining, invalidation, and resize handling.
- Implement deterministic `OnStart`, `OnStop`, and optional `OnTick` boundaries.
- Implement attach, detach, measure, resize, paint, focus, blur, close-request, and closed transitions.
- Implement z-order, hit-testing, focus traversal, pointer capture, modal roots, and raw fallback handlers.
- Implement the action registry, reserved command range, shortcuts, synchronous activation, and bound-control propagation.
- Implement typed single-handler event properties with `TuiChangeOrigin`.
- Add `TuiCustomView` with pure measurement and clipped paint events.

Completion contracts: [event-loop.md](event-loop.md),
[events-and-actions.md](events-and-actions.md), and [view-lifecycle.md](view-lifecycle.md).

## Phase 5 — First usable controls

- Add a frame or window, label, button, input line, check box, list box, and scroll bar.
- Bind buttons, menus, status items, and shortcuts to reusable actions.
- Add a minimal interactive application and a fully headless equivalent test.
- Require keyboard, mouse where applicable, resize, lifecycle, and screen tests for each control.

## Phase 6 — Full application chrome

Add dialogs, menus, status lines, radio groups, stacked pages, memo/editor controls, text viewers, file selection, and advanced controls only after the earlier contracts remain stable.

Custom layout callbacks and a general application message bus are not prerequisites for this phase
sequence. The language gate listed before Phase 4 is required for its public API.
