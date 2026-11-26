# Table

Use a table for a collection with columns, optional sorting and row selection. The gallery puts avatars and badges directly into cells. The [terminal table](../../tui/data-display/table.md) displays text cells.

![Table in the FPAS gallery](images/table.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.Table;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.Tables as W;

// Messages proposed by a sort header or selectable row.
public type Msg = variant
  // Apply the proposed sort to the model's rows.
  SortBy(Value: W.Sort);
  // Select the row with this stable ID.
  SelectPerson(Id: string);
end variant;

// Builds the table example.
// ## Parameters
// - `Rows`: Rows in the model's display order.
// - `Order`: Current sort from the model.
// ## Returns
// The widget for the application view.
public function View(Rows: array of W.Row of Msg; Order: W.Sort): E.Element of Msg;
begin
  return W.Table of Msg('people', [
    new W.Column('Name', Sortable := true),
    new W.Column('Status')
  ], Rows, Order, Msg.SortBy, Msg.SelectPerson);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Columns` | Provide a heading per column. `Width` defaults to `Fill`, `Grow` to 1, `Align` to `Start`, and `Sortable` to false. |
| `Rows` | Pass the rows in display order with stable IDs and one cell per column. Cell.Text is plain text. Cell.Content holds graphical children. |
| `Order / OnSort` | Both default to null. Pair the current Sort with a callback when using sortable headings. Column -1 means unsorted. |
| `OnSelect` | Optional callback receiving the row ID. The model sets each row's Selected flag. |
| `Height / Striped / Compact / Enabled` | Content / false / false / true. Content fits all rows. |

## Column layout

Use a fixed `Width` to reserve logical units for a column. Use `Percent`
to reserve a share of the inner width. Columns with `Fill` share the
remaining width according to `Grow`. The GUI table treats `Content` as `Fill`
for a column width.

When the table is too narrow for its columns, CUI scrolls the columns
horizontally. Use a horizontal wheel gesture or hold Shift while turning
a vertical wheel. CUI fades an edge with more columns beyond it. If `Height`
is `Content`, the table fits all rows. Set a viewport height to scroll the
rows vertically while keeping the heading row visible.

Column `Start` and `End` follow the surrounding reading direction. `Left`
and `Right` select physical edges. `Right` also enables tabular digits in
plain text cells. A cell with graphical children uses a row to lay them out.

## Interaction

Activate a sortable heading to propose a column and direction. Sort the rows
in the update and pass them back with the new sort. Keep each row ID stable
across sorting so focus follows the row. Store selection by row ID in the model
to preserve the selected record. Use `HeaderId(Id, Index)` to address a heading
for focus.

Use a data grid to build rows of a large collection on demand. The row
factory maps each display position to its record and returns a stable row ID.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_table)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/Tables.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/Widgets/People.fpas)
- [Gallery update](../../../../examples/ui/gui/gallery/Pages/Lists/Update.fpas)
- [Virtual list](virtual-list.md)
- [Data grid](../data-grid/data-grid.md)
- [Data grid API](../../widgets.md#data-grid-filter-bar-and-tree-table)
- [Sizing](../../layout/sizing.md)
- [Data display](README.md)

## Source files

- [PeopleTable.fpas](../../../../examples/ui/gui/gallery/Shared/PeopleTable.fpas) supplies the sample people, headings and stable sort to both galleries.

- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [Table.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/Table.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.Table.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.Table.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.Table.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.Table.Types.cs)
  - [CuiWindow.Table.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.Table.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [table.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/table.c)
  - [table.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/table.h)
- `tests/Fixtures/gui-host/`
  - [GalleryTableContract.fpas](../../../../tests/Fixtures/gui-host/GalleryTableContract.fpas)
