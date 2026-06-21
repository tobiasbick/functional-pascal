# Retained controls

`Std.Tui` exposes native label, button, input-line, checkbox, and radio-group views. The host paints
them with the built-in dialog palette and integrates them with retained focus, clipping, commands,
mouse input, keyboard input, and paste.

## Construction

All constructors start with `(App, X, Y, Width, Height)` and return `ViewId`:

| Call suffix | Model arguments |
| ----------- | --------------- |
| `HostCreateLabelView` | `Text`, `Accelerator: Option of string` |
| `HostCreateButtonView` | `Caption`, `CommandId: Option of integer`, `IsDefault` |
| `HostCreateInputLineView` | `Text` |
| `HostCreateCheckBoxView` | `Label`, `Accelerator`, `CommandId`, `Checked` |
| `HostCreateRadioGroupView` | `Options: array of RadioOption` |

`RadioOption` contains `label`, `accelerator`, `commandId`, and `enabled`. Labels are not selectable;
the other four controls are selectable Tab stops. Parent them with `HostSetViewParent` when used in
a dialog subtree.

## Input and commands

- Buttons activate with left click, Enter, or Space.
- Input lines accept character keys, Left/Right/Home/End, Backspace/Delete, and bracketed paste.
- Checkboxes toggle with left click, Enter, or Space.
- Radio groups move the focused option with arrow keys and select with Enter/Space; clicking a row
  selects it directly.
- Optional command ids invoke `ApplicationHandlers.OnCommand` after activation.

## State

| Call | Result/effect |
| ---- | ------------- |
| `QueryInputLineState` | `InputLineState(text, cursor, scrollOffset)` |
| `QueryCheckBoxState` | `CheckBoxState(checked)` |
| `QueryRadioGroupState` | `RadioGroupState(selectedIndex, focusedIndex)`; `-1` means none |
| `HostSetInputLineText` | Replaces text and clamps cursor |
| `HostSetCheckBoxChecked` | Sets checked state |
| `HostSetRadioGroupSelected` | Selects an enabled zero-based option |

## Implementation (contributors)

| Concern | Location |
| ------- | -------- |
| Models/paint | `crates/fpas-std/src/tui/widget/control/` |
| VM bridge/input | `crates/fpas-vm/src/vm/execute/io/tui/control_model/` |
| FPAS regression | `tests/tui/tui_controls_test.fpas` |

## See also

- [Views and focus](views.md)
- [Native testing](testing.md)
- [`Std.Tui` application](README.md)
