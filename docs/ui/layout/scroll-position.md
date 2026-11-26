# Scroll position

`Elements.ScrollView` clips content to a viewport. Use a stable ID so the host can identify that viewport across frames.

## Choose who owns the position

Omit `Position` and `OnScroll` to let the host own the scroll offset. To use a model-owned position, pass both arguments. `Position` supplies the vertical position and `OnScroll` maps a scroll proposal to a message.

| `Layout.ScrollPosition` | Requested position |
| --- | --- |
| `Start` | The start of the content. |
| `End` | The end of the content. The viewport follows the end when content grows. |
| `Offset(Value)` | The distance from the start, limited to the available range. |
| `Reveal(ControlId)` | The smallest vertical move that shows a named child. Available in the GUI. |

The position uses the current scroll range. A change in the range can clamp the offset without sending a message.

## Handle scroll proposals

`Layout.ScrollChange` contains the proposed `Position`, the `Offset` and the `Maximum` offset. The GUI host reports distances in `Measure.Points`. The TUI host reports distances in `Measure.Cells`.

To accept a proposal, store `ScrollChange.Position` in the model and pass it to the next view. Store an `End` position to keep following the end. Use `Offset` to keep a particular distance from the start.

## See also

- [GUI scroll view and reveal](../gui/layout/scroll-view.md)
- [TUI scroll view and keyboard controls](../tui/layout/scroll-view.md)
- [Layout measures](sizing.md#measures)

## Source files

- [Layout.fpas](../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../src/Fpas.Ui/Elements.fpas)
- [Scroll validation](../../../src/Fpas.Ui/Validation.fpas)
- [GUI presentation](../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [TUI scroll navigation](../../../src/Fpas.Ui.Tui/Internal/Hosting/ScrollNavigation.fpas)
