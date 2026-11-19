# Data display

Keep the displayed order, selection and expansion in the model. For a
large collection of equal-height rows, use a virtual list with a row factory.
The [terminal table](../../tui/data-display/table.md) shares the text-cell API.

| Widget | When to use it |
| --- | --- |
| [Table](table.md) | Display rows under sortable headings and compose widgets inside cells. |
| [Tree](tree.md) | Present nested items with expandable branches. |
| [List item](list-item.md) | Present a row with an optional avatar, subtitle and trailing caption. |
| [Reorderable list](reorderable-list.md) | Let the user change the order by dragging. |
| [Accordion](accordion.md) | Group expandable sections such as an FAQ. |
| [Virtual list](virtual-list.md) | Build only the rows needed by a scrolling viewport. |

## See also

- [UI handbook](../README.md)
- [Data display in the gallery](../../../../examples/ui/gui/gallery/Pages/Lists/View.fpas)
- [Data grid](../data-grid/README.md)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Lists/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Lists/Update.fpas)
- [collection_regressions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/collection_regressions.c)
- [CuiCollectionTrackingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiCollectionTrackingTests.cs)
