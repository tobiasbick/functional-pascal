# Data grid

Use a data grid to show rows built when they become visible. The gallery shows
2,000 people with sortable columns, editable names and composed score cells.
Available in **GUI**.

![People data grid in the FPAS gallery](images/data-grid.png)

## Example

```pascal
unit Docs.Ui.DataGrid.DataGrid;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Collections.DataGrids as W;

// Builds a virtual grid for two text columns.
// ## Parameters
// - `Count`: Number of rows in model order.
// - `BuildRow`: Pure factory supplying a stable row ID and two cells.
// - `Current`: Selection, ordering, cursor, anchor and optional vertical target.
// ## Returns
// The grid with proposed state changes as application messages.
public function View(Count: int; BuildRow: function(int): W.Row of W.Value;
  Current: W.Value): E.Element of W.Value;
begin
  return W.DataGrid of W.Value('people',
    [new W.Column('Name', 200.0F, Sortable := true), new W.Column('Team', 140.0F)],
    Count, BuildRow, Current, function(Value: W.Value): W.Value => Value,
    new W.Options(Height := L.Extent.Fixed(L.Measure.Logical(420))));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Count / BuildRow` | Pass the total count and a pure row factory. Native layout asks for visible rows and some extra rows around them. |
| `Columns` | Supply one to 32 columns. Each column has a heading, initial width, alignment, and options for sorting, resizing, and hiding. |
| `Cell` | Supply one Text, Content or Editable cell per column. Editable capacity is at most 256 UTF-8 bytes including the terminator. |
| `Selected / Order` | Both null. Provide a selection mask or sort record to enable each feature. |
| `Cursor / Anchor` | Both minus one. When set, they index the displayed order. Setting a cursor reveals its row. An explicit scroll target takes precedence over the first cursor reveal. |
| `Scroll` | Null for native scrolling. Supply a nonnegative vertical target in logical points to restore it and receive changes through the grid's proposal mapper. |
| `Width / Height` | `Fill`. CUI treats `Content` as `Fill` for both dimensions. |
| `RowHeight` | 40 logical units. The minimum is one logical unit. |
| `Striped / HideCheckboxes / Enabled` | False / false / true. Hiding the checkboxes keeps row selection available. |
| `OnRowClick / OnDoubleClick` | Both null. Each callback receives the clicked row's display index. |

## Column and viewport layout

Give each column an initial `Width` in logical units. CUI uses 150 logical
units when `Width` is zero. Drag a column divider to resize the column. Set
`Fixed` to true to disable this drag. CUI gives any remaining width to the
last visible column.

The grid retains resized widths by column position. Appended columns receive
their initial widths.

Use a `Priority` from 1 through 255 for a column that can hide. If the
viewport narrows, CUI hides the largest positive priorities first. A priority
of zero keeps the column visible, and CUI keeps at least one column. Supply
one cell for each column even when CUI hides some columns. If the visible
columns still overflow, scroll horizontally to reach them.

Column alignment defaults to `Start`. Use `End` for the trailing edge, or
`Left` and `Right` for physical edges. CUI keeps the header visible when
the rows scroll vertically.

## Interaction

Click sortable headings to propose an order. Shift-click selects a range and
Ctrl-click toggles one row. Store selections by stable row ID when sorting or
filtering changes their positions.

Use the arrows, Page Up, Page Down, Home or End to move the keyboard cursor.
Space toggles selection. Ctrl+A selects all rows and Escape clears selection.
A checkbox click changes the selection and sets the origin for the next
Shift range.

`OnRowClick` observes clicks on the row itself. The grid sends any state
proposal first. The row callback can report a repeated click when the grid
state stays the same.

### Editing

Use `Cell.Editable` to draw single-line text with an optional leading array
of elements, such as an avatar. Pass a capacity from 2 through 256 bytes
including the UTF-8 terminator. The model caption must fit in that capacity.

Double-click the text to start a draft. Enter on the focused grid row opens
its first visible editable cell. Enter in the editor sends the draft to
`OnCommit`. Moving focus out of the editor sends a changed draft. Store
the proposed text in the model to accept it.

Escape discards the draft. The grid also discards the draft when the row
leaves the realized range or changes identity. Hiding the column, disabling
or removing the grid, changing the capacity or removing the editor discards
the draft. Undo applies within the current draft.

Declare open modal scopes outside the row factory. The GUI validates realized
`Content` children and `Editable` leading children before drawing them and
rejects open dialogs, sheets, command palettes and file pickers there.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_data_grid_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/DataGrids.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [Gallery update function](../../../../examples/ui/gui/gallery/Pages/DataGrid/Update.fpas)
- [Table](../data-display/table.md)
- [Responsive layout](../../layout/responsive.md)
- [Data grid](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [DataGrid.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/DataGrid.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.DataGrid.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.DataGrid.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.DataGrid.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.DataGrid.Types.cs)
  - [CuiWindow.DataGrid.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.DataGrid.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [grid_editing.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/grid_editing.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [data_grid.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/data_grid.c)
  - [data_grid.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/data_grid.h)
- `tests/Fixtures/gui-host/`
  - [GalleryGridRenameContract.fpas](../../../../tests/Fixtures/gui-host/GalleryGridRenameContract.fpas)
