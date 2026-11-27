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

## Data display gallery page

The Table section shows five people. Name, Role and Status are sortable.
The table keeps the selected person when the sort order changes.

The Tree view and List cards share a [responsive grid](../feedback/grid.md).
The grid fits up to two columns with a minimum width of 260 logical units
and a gap of 16. Each column uses a 12-unit gap between the heading and card.

The Reorderable list shows five tasks with status badges. Dropping a task
updates the order and shows the Order saved toast. The Accordion card has
three independent answers. The first answer starts open.

The Virtual list contains ten thousand invoices. Its 220-unit viewport
builds visible rows of 36 logical units. The label, amount, and status are
computed from the row index.

## See also

- [UI handbook](../README.md)
- [Data grid](../data-grid/README.md)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Lists/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Lists/Update.fpas)
- [View.fpas](../../../../examples/ui/gui/gallery/Pages/Lists/View.fpas)
- [collection_regressions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/collection_regressions.c)
- [CuiCollectionTrackingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiCollectionTrackingTests.cs)
