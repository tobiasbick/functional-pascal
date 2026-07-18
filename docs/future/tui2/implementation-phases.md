# Std.Tui2 implementation phases

## Phase 1 — Geometry, text, cells, and canvas

The geometry, color, style, cell-value, palette, and display-width baseline is documented in the
current [`Std.Tui2` reference](../../pascal/std/tui2/README.md) and
[`Std.Console` cell reference](../../pascal/std/console/cells-frames.md).

- Implemented: console cells accept one renderable grapheme cluster and preserve wide-glyph continuation invariants.
- Implemented: clipping and the headless cell surface.
- Implemented: the transient `TuiCanvas` drawing boundary, including grapheme-aware `WriteText`.

Completion contracts: [geometry.md](geometry.md), [text-and-cells.md](text-and-cells.md), and the pure-value section of [testing.md](testing.md).

## Phase 2 — Application registry and runtime safety

- **Partial foundation.** Headless applications, actions, and buttons use application-scoped
  registry slots, generation checks, cross-application validation, deterministic destruction, and
  application-close cleanup. Custom views and generic layouts use the same model, including reusable
  slots. Views also retain bounds, visibility, and enabled state. Headless containers own child
  subtrees and one root layout, and each headless application may create one desktop root. Headless
  `Post` callbacks drain FIFO before and after `OnTick`.
- Implement the desktop root, parent-child ownership, destruction, stale-handle diagnostics, and tags.
- Extend `OpenForTest` to the generic view registry and add transactional interactive terminal acquisition.
- Add the VM terminal-mode restoration safety net.
- Extend `Post` from headless scheduling to the generic typed main-task queue and enforce
  main-task-only UI mutation.

Completion contracts: [handles-and-ownership.md](handles-and-ownership.md), [runtime-boundary.md](runtime-boundary.md), and the registry/failure sections of [testing.md](testing.md).

## Phase 3 — Layout engine

- **Implemented foundation.** `TuiMeasureConstraint`, `TuiMeasureSpec`, and `TuiMeasureResult`
  provide validated pure values. Views retain a size hint and independent size policy in registry
  state. Recursive layout measurement calculates minimum, preferred, and maximum sizes from visible
  views, nested layouts, spacers, spacing, and margins. Width-dependent control measurement remains.
- **Implemented foundation.** `TuiSizePolicy`, `TuiMargins`, `TuiAlignment`, `TuiSpacer`, and
  `TuiLayoutItem` are implemented as validated values. `TuiLayoutItems` provides ordered lists,
  exclusive root-or-nested ownership, cycle rejection, stable removal, and nested destruction.
  Immutable layout kinds select the algorithm; live `TuiLayoutSettings` applies margins and spacing.
- **Implemented:** typed horizontal, vertical, grid, form, and stacked layouts; recursive headless
  allocation; alignment; stretch/expanding slot growth; finite item maximum enforcement; grid spans;
  form rows; stable stacked measurement; current-page selection; overlap rejection; and deterministic
  item or track remainders.
- **Implemented:** coalesced invalidation from layout items, view measurement inputs, settings, and
  nested layouts; explicit container and desktop layout passes; and container resize detection.
- **Implemented:** terminal-too-small detection with minimum, available, and per-axis overflow
  extents while preserving minimum child geometry.
- **Implemented:** headless `TuiScrollView` viewport sizing, preferred content extent, clamped
  two-axis offsets, resize/content invalidation, and offset layout allocation.

Completion contract: [layout.md](layout.md) and the layout section of [testing.md](testing.md).

## Phase 4 — Lifecycle, event routing, and actions

- **Next:** implement the bounded application loop, posted callback draining, invalidation, and
  resize handling.
- **Partial foundation.** Headless application lifecycle events, live action state,
  `TuiAction.OnExecute`, button action binding, and action-before-`OnClick` dispatch are implemented.
- Integrate the existing lifecycle events with the bounded loop and terminal lifecycle.
- Implement attach, detach, measure, resize, paint, focus, blur, close-request, and closed transitions.
- Implement z-order, hit-testing, focus traversal, pointer capture, modal roots, and raw fallback handlers.
- Extend the existing action registry with shortcuts and propagation to every bound-control type.
- Implement typed single-handler event properties with `TuiChangeOrigin`.
- Add `TuiCustomView` with pure measurement and clipped paint events.

Completion contracts: [event-loop.md](event-loop.md),
[events-and-actions.md](events-and-actions.md), and [view-lifecycle.md](view-lifecycle.md).

## Phase 5 — First usable controls

- Add a frame or window, label, input line, check box, list box, and scroll bar; turn the existing
  semantic button into a rendered, focusable control.
- Bind buttons, menus, status items, and shortcuts to reusable actions.
- Add a minimal interactive application and a fully headless equivalent test.
- Require keyboard, mouse where applicable, resize, lifecycle, and screen tests for each control.

## Phase 6 — Full application chrome

Add dialogs, menus, status lines, radio groups, memo/editor controls, text viewers, file selection,
and advanced controls only after the earlier contracts remain stable.

Custom layout callbacks and a general application message bus are not prerequisites for this phase
sequence.
