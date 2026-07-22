# Phase 2 — Deterministic layout and arranged-frame paint

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).

## Gate 2.A — Freeze the v1 layout constructor encoding

**Status:** complete.

[api-surface.md](../api-surface.md) now fixes the value records, defaults, Row/Column spacing,
single-child layout wrapper, and spacer encoding. Grid, Form, and Stack are explicitly deferred.
An implementation agent must use that contract and must not expose `TuiLayout` handles.

## Task 2.1 — Add frozen layout values and element variants

**Status:** ready.

**Files:** add one-concern files under `lib/Std/Tui3/Layout/` for the declarations frozen by the
gate; modify `Elements/Element.fpas`, `Elements/Validate.fpas`, and `lib/Std/Tui3.fpas`; add
`tests/stdlib/tui3/layout_values_test.fpas` and extend `element_tree_test.fpas`.

**Contract:** elements remain immutable recursive data returned by `View`; layout settings contain
only values. Existing builders remain the construction facade and are extended rather than
duplicated.

**Done:** every new value validates its invariants, every new element participates in recursive
validation, and no live layout/container API is introduced.

## Task 2.2 — Upgrade intrinsic measurement

**Status:** blocked by Task 2.1.

**Files:** modify `Layout/Measure.fpas`; split combination algorithms into thematic files before
the module exceeds 400 LOC; add `tests/stdlib/tui3/layout_measure_test.fpas`.

**Contract:** implement the minimum/preferred/maximum and policy rules in
[layout.md](../layout.md). Allocation uses display cells, stable child order, and no surface access.
Port algorithms from `lib/Std/Tui2/Layouts/`, not ownership or invalidation code.

**Done:** leaf intrinsic sizes, nested Row/Column, spacing, margins, frame insets, Empty children,
and undersized bounds have exact pure-value assertions.

## Task 2.3 — Build the private arranged-frame index

**Status:** blocked by Task 2.2.

**Files:** add `Layout/Frame.fpas` and `Layout/Arrange.fpas`; modify `Runtime/Frame.fpas`; add
`tests/stdlib/tui3/layout_arrange_test.fpas`.

**Contract:** follow [layout.md](../layout.md)'s arranged-frame contract. Traverse the public
element tree in deterministic preorder and store private frame entries with parent index, resolved
application bounds, and effective clip. Do not copy a `TuiElement` or child array into an entry.
The frame is rebuilt for each painted model/size pair and is not public application state.

**Done:** Row/Column allocation, stable leftover distribution, nested clips, frame insets, resize,
and terminal-too-small fit are asserted. Instrumented traversal shows one arranged entry per
element and no second element tree.

## Task 2.4 — Paint only from arranged geometry

**Status:** blocked by Tasks 1.4 and 2.3.

**Files:** modify `Rendering/Paint.fpas` and `Runtime/Frame.fpas`; add
`tests/stdlib/tui3/layout_snapshot_test.fpas`.

**Contract:** paint consumes the element tree plus its matching arranged frame. It must not call
measurement or recompute child bounds. Back-to-front order and clips come from the frame index;
all writes go through canvases.

**Done:** exact snapshots cover Label, Button, Input, Row, Column, Window, Dialog, Desktop, clipping,
Unicode width, and at least two terminal sizes.

## Task 2.5 — Preserve the arranged frame for routing

**Status:** blocked by Task 2.4.

**Files:** modify `Runtime/Application.fpas`, `Runtime/Frame.fpas`, and `Runtime/Routing.fpas`; add
`tests/stdlib/tui3/arranged_frame_lifetime_test.fpas`.

**Contract:** initial `View` → arrange → paint completes before input. Routing reads exactly the
previous successfully painted tree/frame pair. A new pair replaces it only after the next
successful `View` and arrange. No arranged geometry enters the application model.

**Done:** resize and model changes target the last visible frame deterministically; failed
View/arrange does not leave a mismatched tree/frame pair.

## Phase checkpoint

`View` trees render through measure → arrange → clipped canvas paint; paint does not measure;
routing has one matching previous frame.
