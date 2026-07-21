# Std.Tui3 implementation phases

## Cross-cutting language follow-ups

For every compiler panic or language restriction found while implementing Tui3, add an
entry to [compiler-panic-followups.md](../compiler-panic-followups.md) in the same change.
Do not silently change the language or leave an undocumented workaround.

## Phase 0 — Plans and executable feasibility

- Write and keep `docs/future/tui3/` current.
- Mark `docs/future/tui2/` frozen/superseded.
- No `docs/pascal/std/tui3/` until behavior exists.
- Compile the exact generic `RunIterations<TModel>`, `Update`, mutable `TuiCmd` output, and `View`
  callable pattern from [api-surface.md](api-surface.md).
- Build one headless vertical slice with a recursive tree containing a label, controlled input,
  button, and modal dialog.
- Prove unique control ids, repeatable action ids, focus movement, text changes, activation, initial
  render, FIFO routing, and quit order.
- Run repeated frames at representative tree and terminal sizes. Record timings and clone/allocation
  evidence for tree traversal and surface painting.
- If aggregate cloning dominates, implement shared or copy-on-write VM storage as a prerequisite and
  repeat the spike. Do not introduce public retained view handles as a workaround.

Completion: the slice passes [testing.md](testing.md)'s Phase 0 suite and the plan records the final
compiling signatures. Phase 1 is blocked until this gate passes.

## Phase 1 — Values

Port or reimplement under `lib/Std/Tui3/`:

- Geometry (`Point`, `Size`, `Rect`)
- Cells, styles, palette
- Host-owned working surface, frame-scoped internal canvas, and immutable surface snapshot

Completion: pure-value tests plus surface/snapshot ownership tests; no public mutable cell-grid
value.

## Phase 2 — Elements, layout, TV paint

- `TuiElement` constructors for `None`, layout (`Row`/`Column`/…), `Label`, chrome frames
- Pure `Measure` / `Arrange`
- Paint window/dialog/label into a headless surface
- Snapshot tests for TV-looking frames

Completion: `View`-compatible trees render without input.

## Phase 3 — MVU runtime (headless)

- `TuiMsg`, `TuiCmd`, `TuiControlId`, `TuiAction`
- `OpenForTest`, inject, `RunIterations`
- `Update` / `View` loop with layout + paint each iteration
- Button and controlled input elements emitting source-aware messages
- Focus and message queue order
- `None` / `Quit` command order

Completion: confirm-dialog style demo without Create/Add/Destroy/OnClick and without hidden control
state. No closure-based posting or general asynchronous effect claim.

## Phase 4 — Controls and chrome

- CheckBox, List, Scroll
- MenuBar, StatusLine
- Modal-as-tree input routing
- Focus helpers driven by model fields
- Terminal-too-small overlay

Completion: enough controls for a static small-application chrome sketch, still headless-first.
Movable/resizable overlapping windows, full editor behavior, and a complete Turbo Vision desktop
manager are not implied by this phase.

## Phase 5 — Interactive terminal

- `AcquireInteractiveTerminal` integration
- `Run` entry point
- Mode restoration and failure rollback
- Flush surface through `Std.Console`

Completion: one interactive example application.

## Phase 6 — Production-readiness gate

- Repeat the Phase 0 performance measurements on the complete control set.
- Audit the applications, examples, tests, and workflows that still depend on production `Std.Tui`.
- Port at least one representative real application flow, not only the confirm-dialog slice.
- List every lost production capability explicitly. Implement required gaps or obtain an explicit
  decision that the loss is acceptable.
- If the representative application requires timers, worker results, file dialogs, or other
  effects, design and prove a data-only message transport before promotion.
- Confirm current Tui3 docs and tests cover the API that will be renamed.

Completion: recorded performance evidence, feature-gap audit, representative application, and an
explicit promote decision. A successful demo alone is not sufficient.

## Phase 7 — Promote to `Std.Tui`

Only after Phase 6 passes:

1. Remove `Std.Tui` (turbo-vision bridge), its VM bridge, docs, and tests.
2. Remove `Std.Tui2`, its docs, and tests.
3. Rename `Std.Tui3` → `Std.Tui` (unit, `lib/Std/Tui/`, `docs/pascal/std/tui/`, tests).
4. Delete `docs/future/tui3/` and the Tui2 freeze notice once obsolete.
5. Remove [tui-bridged-readback.md](../tui-bridged-readback.md) with the bridge.
6. Update [`docs/future/README.md`](../README.md) and agent skills that still describe the
   turbo-vision `Std.Tui` bridge as current.

Until Phase 7, do not document Tui3 behavior under `docs/pascal/std/tui/` (that path still
means the turbo-vision facade).

## Explicit non-work

- Finishing remaining Tui2 retained controls for their own sake.
- Declarative wrappers over Tui2 handles.
- React-hooks component state as the primary model.
- Arbitrary closure commands or worker posting in v1.
- Claiming full Turbo Vision desktop behavior from static TV-looking chrome.
- Keeping three public TUI units after a successful promote.
