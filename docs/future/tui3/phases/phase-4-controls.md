# Phase 4 — Controlled controls and application chrome

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).
Each control task owns its enum variant, builder, validation, intrinsic measurement, arrangement,
paint, key routing, pointer routing, dedicated message, facade export, and the four assertion levels
required by [testing.md](../testing.md). Do not combine control tasks.

## Task 4.1 — CheckBox

**Status:** complete.

Implement the exact `CheckBox(Id, Text, Checked, ChangeAction)` contract and
`CheckChanged(Source, Action, Checked)` message from [api-surface.md](../api-surface.md). Add
`check_box_keyboard_test.fpas`, `check_box_pointer_test.fpas`,
`check_box_layout_test.fpas`, and `check_box_snapshot_test.fpas`. Space/click proposes the inverse
model value; the control stores no checked state outside the element.

**Files:** modify `Elements/Element.fpas`, `Elements/Validate.fpas`, `Layout/Measure.fpas`, the
arrange/paint/routing modules, `Runtime/Msg.fpas`, and `lib/Std/Tui3.fpas`; split a module by control
before it would exceed 400 LOC.

## Gate 4.A — Freeze List selection behavior

**Status:** complete.

**Prerequisite:** Task 4.1.

Record in [api-surface.md](../api-surface.md) the valid `Selected` range, empty-list representation,
Up/Down/Home/End behavior, pointer row mapping, and deterministic viewport rule. The contract must
keep selection and any viewport input in model data.

## Task 4.2 — List

**Status:** complete.

Implement `List(Id, Items, Selected, ChangeAction)` and
`SelectionChanged(Source, Action, Selected)` exactly as frozen. Modify the same owning element,
layout, paint, routing, message, and facade modules as Task 4.1. Add separate keyboard, pointer,
layout/resize, and snapshot tests named `list_*_test.fpas`. Routing only proposes a valid next
index.

## Gate 4.B — Freeze Scroll clamping and input

**Status:** complete.

**Prerequisite:** Task 4.2.

Record in [api-surface.md](../api-surface.md) valid model offsets, effective clamping, arrow/page/
wheel increments, and behavior when content is smaller than its viewport. Do not add hidden
scrollbar state.

## Task 4.3 — Scroll

**Status:** complete.

Implement `Scroll(Id, Offset, ChangeAction, Child)` and
`ScrollChanged(Source, Action, Offset)` using the frozen behavior and
[layout.md](../layout.md)'s viewport contract. Modify the owning element, layout, paint, routing,
message, and facade modules. Add `scroll_keyboard_test.fpas`, `scroll_pointer_test.fpas`,
`scroll_layout_test.fpas`, and `scroll_snapshot_test.fpas`. The offset is controlled data; clipping
and hit-testing use the arranged frame.

## Gate 4.C — Freeze MenuBar data and interaction

**Status:** complete.

**Prerequisite:** Task 4.3.

Replace the current `TuiMenuItem` sketch in [api-surface.md](../api-surface.md) with exact compiling
records/enums and constructors. Specify source control ids, actions, disabled items, shortcuts,
keyboard/pointer order, and whether v1 is a flat action bar or has model-controlled open menus. Do
not inherit Turbo Vision command offsets or live menu handles.

## Task 4.4 — MenuBar

**Status:** complete.

Add one focused value file and modify the owning element, layout, paint, routing, message, and
facade modules. Add `menu_bar_keyboard_test.fpas`, `menu_bar_pointer_test.fpas`,
`menu_bar_layout_test.fpas`, and `menu_bar_snapshot_test.fpas`.

## Gate 4.D — Freeze StatusLine data and interaction

**Status:** complete.

**Prerequisite:** Task 4.4.

Replace the current `TuiStatusItem` sketch in [api-surface.md](../api-surface.md) with exact compiling
records/enums and constructors. Specify display-only versus actionable items, source ids, shortcuts,
disabled behavior, and focusability.

## Task 4.5 — StatusLine

**Status:** complete.

Add one focused value file and modify the owning element, layout, paint, routing, message, and
facade modules. Add `status_line_keyboard_test.fpas`, `status_line_pointer_test.fpas` when
actionable, `status_line_layout_test.fpas`, and `status_line_snapshot_test.fpas`. A display-only
status line must not be made focusable.

## Task 4.6 — Focus helpers and terminal-too-small overlay

**Status:** complete.

Add pure helpers that choose valid model focus from the current tree and expose layout-fit results
without retaining focus internally. Add `focus_helpers_test.fpas` and
`terminal_too_small_overlay_test.fpas`. The foremost Dialog remains the modal routing root.

## Phase checkpoint

**Status:** complete.

Every public interactive element has keyboard, applicable pointer, layout/resize, and screen
coverage; the static application chrome sketch runs headlessly. This phase does not imply movable/
resizable overlapping windows, a full editor, or a Turbo Vision desktop manager.

Phase 5 is unblocked.
