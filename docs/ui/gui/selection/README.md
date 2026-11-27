# Selection

Keep the current value in the model. When the user proposes a change, apply
it in the update function and pass the updated value back to the widget.
Checkbox, Switch and Radio group also support the TUI host.

| Widget | When to use it |
| --- | --- |
| [Checkbox](checkbox.md) | Select an independent option, such as receiving email notifications. |
| [Switch](switch.md) | Turn a setting such as Wi-Fi on or off. |
| [Radio button](radio-button.md) | Provide one choice in a layout you compose yourself. |
| [Radio group](radio-group.md) | Offer exclusive choices in a vertical list. |
| [Radio cards](radio-cards.md) | Give each exclusive choice room for a description, icon or badge. |
| [Slider](slider.md) | Adjust one value within a bounded range. |
| [Range slider](range-slider.md) | Adjust a lower and upper endpoint on the same track. |
| [Chip](chip.md) | Toggle an independent category in a wrapping row. |
| [Rating](rating.md) | Choose a whole-star rating. |

## Selection gallery page

The Checkbox and Switch sections share a [responsive grid](../feedback/grid.md).
The grid fits up to two columns with a minimum width of 260 logical points
and a gap of 16. Each section uses a [column](../layout/column.md) with a
12-point gap between its heading and card.

The Checkbox and Switch cards use a gap of 14 logical points. The Radio
group card places three radio rows directly inside with a gap of 12.
The Sliders card uses 20 and the Chips and rating card uses 16.
The Radio cards section uses the widget's own responsive grid.
Six independent chips use a [wrapping row](../layout/row.md) with a gap of 8.

The disabled checkbox reflects the weekly summary setting on Overview.
The dialog and notifications sheet share the Wi-Fi value. The sheet also
shares the Bluetooth value. The minimum score in the filter popover and
the brightness slider share a value. The brightness slider uses ten-point
steps and displays a percentage.

## See also

- [UI handbook](../README.md)
- [Segmented control](../actions/segmented-control.md)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Selection/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Selection/Update.fpas)
- [View.fpas](../../../../examples/ui/gui/gallery/Pages/Selection/View.fpas)
- [SelectionContract.fpas](../../../../tests/Fixtures/gui-host/SelectionContract.fpas)
