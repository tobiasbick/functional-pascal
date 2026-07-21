# Std.Tui2 implementation phases

## Cross-cutting compiler and language follow-ups

For every compiler panic or language restriction encountered while implementing Tui2, add a
source-shape, workaround, and later-resolution entry to
[compiler-panic-followups.md](../compiler-panic-followups.md) in the same change. This includes
limitations that can be avoided locally but could be improved in FPAS later. Do not silently change
the language or leave an undocumented workaround in Tui2 source.

## Phase 1 — Geometry, text, cells, and canvas

The geometry, color, style, cell-value, palette, and display-width baseline is documented in the
current [`Std.Tui2` reference](../../pascal/std/tui2/README.md) and
[`Std.Console` cell reference](../../pascal/std/console/cells-frames.md).

- Implemented: console cells accept one renderable grapheme cluster and preserve wide-glyph continuation invariants.
- Implemented: clipping and the headless cell surface.
- Implemented: the clipped `TuiCanvas` drawing value, including grapheme-aware `WriteText`.

Completion contracts: [geometry.md](geometry.md), [text-and-cells.md](text-and-cells.md), and the pure-value section of [testing.md](testing.md).

## Phase 2 — Application registry and runtime safety

- **Implemented headless core.** Applications, actions, views, layouts, and controls use
  application-scoped generational registries with cross-application validation, deterministic
  destruction, application-close cleanup, one desktop root, retained view state, subtree ownership,
  and FIFO post callbacks before and after `OnTick`.
- **Implemented interactive terminal ownership.** `Std.Console.AcquireInteractiveTerminal` /
  `ReleaseInteractiveTerminal` provide transactional mode ownership with exclusive acquire,
  reverse rollback, and console Drop restoration of owned screen modes. `TuiApplication.Open`
  acquires the session; `Close` releases it. Worker-task UI restrictions remain open.

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

- **Implemented headless core:** bounded application iterations, automatic start, orderly quit,
  posted callback boundaries, dirty desktop layout completion, retained application size, and
  desktop resize propagation.
- **Implemented interactive loop:** `TuiApplication.Open` / `Run` acquire the console session,
  flush painted surfaces through `BeginFrame` / `WriteCells` / `Present`, poll
  `ReadEventTimeout`, map key and pointer events into the existing input queue (with 1-based to
  0-based pointer conversion), apply resize events, and release the terminal on close.
- **Partial foundation.** Headless application lifecycle events, live action state,
  `TuiAction.OnExecute`, button action binding, and action-before-`OnClick` dispatch are implemented.
- Integrate remaining lifecycle diagnostics with worker-task UI restrictions.
- **Implemented headless core:** typed custom views with synchronous attach, structural detach before
  release, coalesced post-layout resize delivery, and callback revalidation.
- **Implemented headless core:** custom-view measurement integrated with measure specs and size
  policies, explicit paint invalidation, local paint coordinates, ancestor clipping, retained
  headless surfaces, and paint delivery after resize and before `OnTick`.
- **Implemented headless core:** one focus owner per application, eligibility through the attached
  visible and enabled ancestor chain, ordered blur-before-focus notification, deferred reentrant
  focus requests, focus-aware paint context, and synchronous repair after eligibility changes.
- **Implemented headless core:** vetoable custom-view `Close`, ordered blur/detach/closed delivery,
  live sender inspection during `OnClosed`, non-veto ownership destruction, and container ownership
  cycle rejection.
- **Implemented headless core:** stable container Z-order with subtree ordering, central tree-ordered
  custom-view painting, topmost hit-testing through the resolved paint clip, and wrapped forward or
  backward focus traversal with effective visibility and enabled-state filtering.
- **Implemented headless core:** canonical console key values, normalized pointer values, FIFO test
  injection, one-input-per-iteration routing, focused and topmost custom-view handlers, pointer
  capture, nested modal roots, focus restoration, and application fallback handlers.
- **Implemented headless core:** action shortcuts with exact key and modifier matching, deterministic
  creation-order conflict resolution, and routing after an unconsumed focused custom-view key handler
  but before the application fallback.
- **Implemented headless core:** retained, attachable buttons with focus traversal, hit testing,
  Enter, Space, and left-pointer-down activation through the existing action-before-`OnClick` order.
- **Implemented headless core:** retained button painting with normal, focused, and disabled roles,
  text invalidation, and central tree-ordered composition with custom views.
- **Implemented headless core:** intrinsic retained-button measurement from bracketed grapheme text,
  with layout invalidation after a text replacement.
- **Implemented headless core:** attachable labels with grapheme-aware fixed measurement, retained
  normal/disabled painting, text replacement invalidation, and generational cleanup.
- **Implemented headless core:** retained input lines built on the custom-view input and focus path,
  with bounded single-line editing, pointer cursor placement, fixed measurement, and retained
  normal/focused/disabled painting.
- **Implemented headless core:** `TuiChangeOrigin` and the typed single-handler
  `TuiInputLine.OnChanged` event. Property changes report `Programmatic`, accepted edits report
  `User`, and unchanged values do not raise an event.
- **Implemented headless core:** retained check boxes with grapheme-aware text measurement,
  keyboard and pointer toggling, normal/focused/disabled painting, and typed `OnChanged` events.
- **Implemented headless core:** retained list boxes with grapheme-aware item measurement,
  clamped selection, keyboard and pointer navigation, normal/focused/selected/disabled painting,
  layout invalidation, and typed `OnSelectionChanged` events.
- **Implemented headless core:** retained frames with one-cell content insets, inner layout bounds,
  child clipping, frame/disabled painting, and subtree ownership.
- **Implemented headless core:** titled retained windows with the same inner container contract,
  title repainting, disabled painting, and subtree ownership.
- **Implemented headless core:** vertical scroll bars with clamped range state, semantic position
  events, keyboard/pointer control, and deterministic thumb painting.
- **Implemented headless core:** titled retained dialogs with explicit modal opening and closing,
  modal routing isolation, focus restoration, inner layout bounds, subtree ownership, and explicit
  command completion.
- **Implemented headless core:** check boxes can bind a same-application action, share its checked
  state on binding and toggle, execute it before `OnChanged`, and reject user toggles while the
  action is disabled.
- Enforce measurement and paint mutation restrictions at runtime. Limiting canvas use to an active
  paint callback is deferred until FPAS can represent an opaque transient handle; see
  [compiler-panic-followups.md](../compiler-panic-followups.md).

Completion contracts: [event-loop.md](event-loop.md),
[events-and-actions.md](events-and-actions.md), and [view-lifecycle.md](view-lifecycle.md).

## Phase 5 — First usable controls

The implemented retained controls have deterministic headless keyboard, pointer, layout, lifecycle,
and screen coverage. Interactive `Open` / `Run` and terminal flush are implemented. Remaining work
is a small end-to-end demo application that exercises the live loop with real controls, plus any
matching headless regression that locks the same scenario.

## Phase 6 — Full application chrome

Add menus, status lines, radio groups, memo/editor controls, text viewers, file selection, and
advanced controls only after the earlier contracts remain stable.

Custom layout callbacks and a general application message bus are not prerequisites for this phase
sequence.
