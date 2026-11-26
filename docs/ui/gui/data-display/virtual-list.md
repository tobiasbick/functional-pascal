# Virtual list

Use a virtual list to show a large collection with rows of equal height.
The gallery uses a row factory for 10,000 invoices. Available in **GUI**.

![Virtual list in the FPAS gallery](images/virtual-list.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.VirtualList;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Collections.VirtualLists as W;

// Builds the virtual list example.
// ## Parameters
// - `Count`: Number of available rows.
// ## Returns
// The widget for the application view.
public function View(Count: int): E.Element of string;
begin
  return W.VirtualList of string('invoices', Count, 36.0F,
    L.Extent.Fixed(L.Measure.Logical(220)),
    function(Index: int): E.Element of string => E.Text of string('Invoice ' + (Index + 1).ToString()));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Count` | Nonnegative row count in the current model. |
| `RowHeight` | Fixed height of at least one logical unit. The total row height must be less than `int.MaxValue`. |
| `Height` | Viewport extent. CUI treats `Content` as `Fill`. |
| `BuildRow` | A pure function that builds a row from its zero-based index. |
| `Offset` | Null or a finite, nonnegative scroll offset in logical units. The host restores the offset when the view appears. |
| `OnScroll` | Null or a callback that maps a native scroll offset to a message. |

## Interaction

Scrolling changes which rows CUI requests. CUI can include rows just outside
the viewport. Give interactive children unique, stable IDs across the view.
The GUI snapshot reports the realized range as `first:last`, where `last` is
the first index outside the range.

The list retains its native offset while it stays in the view. Store offset
messages in the model to restore the position when the list reappears. If
the row count shrinks, CUI clamps the offset to the new range.

Declare open modal scopes outside `BuildRow`. The GUI host validates realized
rows before drawing them. It rejects open dialogs, sheets, command palettes,
and file pickers in those rows.

The host rejects a nested list when CUI has reached its layout or ID capacity,
leaving the enclosing list and row available for the rest of the frame.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/VirtualLists.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/Widgets/Invoices.fpas)
- [Table](table.md)
- [Scroll view](../layout/scroll-view.md)
- [Data display](README.md)
- [Collection scope tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiCollectionTrackingTests.cs)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [VirtualList.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/VirtualList.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.VirtualList.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.VirtualList.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.VirtualList.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.VirtualList.Types.cs)
  - [CuiWindow.VirtualList.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.VirtualList.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [virtual_list.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/virtual_list.c)
  - [virtual_list.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/virtual_list.h)
