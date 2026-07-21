# Std.Tui3 implementation phases

## Cross-cutting language follow-ups

For every compiler panic or language restriction found while implementing Tui3, add an
entry to [compiler-panic-followups.md](../compiler-panic-followups.md) in the same change.
Do not silently change the language or leave an undocumented workaround.

## Phase 0 — Plans (this directory)

- Write and keep `docs/future/tui3/` current.
- Mark `docs/future/tui2/` frozen/superseded.
- No `docs/pascal/std/tui3/` until behavior exists.

## Phase 1 — Values

Port or reimplement under `lib/Std/Tui3/`:

- Geometry (`Point`, `Size`, `Rect`)
- Cells, styles, palette
- Surface and canvas

Completion: pure-value tests; no MVU yet.

## Phase 2 — Elements, layout, TV paint

- `TuiElement` constructors for `None`, layout (`Row`/`Column`/…), `Label`, chrome frames
- Pure `Measure` / `Arrange`
- Paint window/dialog/label into a headless surface
- Snapshot tests for TV-looking frames

Completion: `View`-compatible trees render without input.

## Phase 3 — MVU runtime (headless)

- `TuiMsg`, `TuiCmd`, `TuiAction`
- `OpenForTest`, inject, `RunIterations`
- `Update` / `View` loop with layout + paint each iteration
- Button and input elements emitting actions
- Quit and post drains

Completion: confirm-dialog style demo without Create/Add/Destroy/OnClick.

## Phase 4 — Controls and chrome

- CheckBox, List, Scroll
- MenuBar, StatusLine
- Modal-as-tree input routing
- Focus helpers driven by model fields
- Terminal-too-small overlay

Completion: enough controls for a small IDE-like chrome sketch, still headless-first.

## Phase 5 — Interactive terminal

- `AcquireInteractiveTerminal` integration
- `Run` entry point
- Mode restoration and failure rollback
- Flush surface through `Std.Console`

Completion: one interactive example application.

## Phase 6 — Promote to `Std.Tui`

When the model is accepted:

1. Remove `Std.Tui` (turbo-vision bridge), its VM bridge, docs, and tests.
2. Remove `Std.Tui2`, its docs, and tests.
3. Rename `Std.Tui3` → `Std.Tui` (unit, `lib/Std/Tui/`, `docs/pascal/std/tui/`, tests).
4. Delete `docs/future/tui3/` and the Tui2 freeze notice once obsolete.
5. Remove [tui-bridged-readback.md](../tui-bridged-readback.md) with the bridge.
6. Update [`docs/future/README.md`](../README.md) and agent skills that still describe the
   turbo-vision `Std.Tui` bridge as current.

Until Phase 6, do not document Tui3 behavior under `docs/pascal/std/tui/` (that path still
means the turbo-vision facade).

## Explicit non-work

- Finishing remaining Tui2 retained controls for their own sake.
- Declarative wrappers over Tui2 handles.
- React-hooks component state as the primary model.
- Keeping three public TUI units after a successful promote.
