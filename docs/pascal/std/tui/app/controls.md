# Retained controls

`Std.Tui` exposes native label, button, input-line, memo, checkbox, radio-group, list-box, scroll-bar, and
scroll-view controls. The host paints them with the built-in dialog palette and integrates them with
retained focus, clipping, commands, mouse input, keyboard input, and paste.

## Construction

All constructors start with `(App, X, Y, Width, Height)` and return `ViewId`:

| Call suffix | Model arguments |
| ----------- | --------------- |
| `HostCreateLabelView` | `Text`, `Accelerator: Option of string` |
| `HostCreateButtonView` | `Caption`, `CommandId: Option of integer`, `IsDefault` |
| `HostCreateInputLineView` | `Text` |
| `HostCreateMemoView` | `Text` (multiline, `\n` separated) |
| `HostCreateCheckBoxView` | `Label`, `Accelerator`, `CommandId`, `Checked` |
| `HostCreateRadioGroupView` | `Options: array of RadioOption` |
| `HostCreateListBoxView` | `Items: array of ListBoxItem` |
| `HostCreateScrollBarView` | `ContentLength`, `ViewportLength`, `Vertical` |
| `HostCreateScrollView` | `Lines: array of string` |

`RadioOption` contains `label`, `accelerator`, `commandId`, and `enabled`. Labels are not selectable;
the other controls are selectable Tab stops. Parent them with `HostSetViewParent` when used in
a dialog subtree.

## Input and commands

- Buttons activate with left click, Enter, or Space.
- Input lines accept character keys, Left/Right/Home/End, Backspace/Delete, and bracketed paste.
- Memos accept character keys, Enter for new lines, arrow keys with Shift for selection, PageUp/PageDown
  for vertical scroll, Backspace/Delete, and bracketed paste. An integrated vertical scroll bar appears
  when line count exceeds the view height.
- Checkboxes toggle with left click, Enter, or Space.
- Radio groups move the focused option with arrow keys and select with Enter/Space; clicking a row
  selects it directly.
- Optional command ids invoke `ApplicationHandlers.OnCommand` after activation.
- List boxes use Up/Down/Home/End, Enter/Space, direct row clicks, and mouse-wheel scrolling.
- Scroll bars and scroll views use Up/Down/Home/End, PageUp/PageDown, mouse-wheel scrolling, and
  scroll-bar arrow/track clicks. Dragging the thumb captures the pointer until button release.
  Scroll views reserve one column for an integrated vertical bar when content overflows.

## State

| Call | Result/effect |
| ---- | ------------- |
| `QueryInputLineState` | `InputLineState(text, cursor, scrollOffset)` |
| `QueryMemoState` | `MemoState(text, cursorLine, cursorColumn, scrollOffset, selectionAnchorLine, selectionAnchorColumn)`; anchor fields are `-1` when no range is active |
| `QueryCheckBoxState` | `CheckBoxState(checked)` |
| `QueryRadioGroupState` | `RadioGroupState(selectedIndex, focusedIndex)`; `-1` means none |
| `HostSetInputLineText` | Replaces text and clamps cursor |
| `HostSetMemoText` | Replaces memo text and resets cursor, selection, and scroll |
| `HostSetCheckBoxChecked` | Sets checked state |
| `HostSetRadioGroupSelected` | Selects an enabled zero-based option |
| `QueryListBoxState` | `ListBoxState(selectedIndex, scrollOffset)`; `-1` means no enabled row |
| `HostSetListBoxItems` | Replaces rows and resets selection/scroll |
| `HostSetListBoxSelected` | Selects and reveals an enabled row |
| `QueryScrollBarState` | `ScrollBarState(scrollOffset, contentLength, viewportLength)` |
| `HostSetScrollBarExtents` | Replaces logical scroll extents and clamps offset |
| `QueryScrollViewState` | `ScrollViewState(scrollOffset, lineCount)` |
| `HostSetScrollViewLines` | Replaces lines and resets scroll |

## Implementation (contributors)

| Concern | Location |
| ------- | -------- |
| Models/paint | `crates/fpas-std/src/tui/widget/control/` |
| VM bridge/input | `crates/fpas-vm/src/vm/execute/io/tui/control_model/` |
| FPAS regression | `tests/tui/controls/tui_controls_test.fpas`, `tests/tui/controls/tui_memo_test.fpas`, `tests/tui/controls/tui_list_box_test.fpas`, `tests/tui/controls/tui_scroll_bar_test.fpas`, `tests/tui/controls/tui_scroll_view_test.fpas` |

## See also

- [Views and focus](views.md)
- [Native testing](testing.md)
- [`Std.Tui` application](README.md)
