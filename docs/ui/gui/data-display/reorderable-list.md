# Reorderable list

Use a reorderable list to let the user arrange a small set of items by dragging. Available in **GUI**.

![Reorderable list in the FPAS gallery](images/reorderable-list.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.ReorderableList;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.ReorderableLists as W;

// Builds the reorderable list example.
// ## Parameters
// - `Order`: Current display order from the model.
// ## Returns
// The widget for the application view.
public function View(Order: array of int): E.Element of array of int;
begin
  return W.ReorderableList of array of int('task-order', [
    [E.Text of array of int('Write the release notes')],
    [E.Text of array of int('Update the screenshots')],
    [E.Text of array of int('Tag the release')]
  ], Order, function(Proposed: array of int): array of int => Proposed);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | A stable list ID. |
| `Items` | Up to 256 rows in canonical index order. Each row is an array of direct child elements. |
| `Order` | A permutation of the zero-based item indices. Use [0, 1, 2] for the example's initial order. |
| `OnChange` | Maps the proposed order to an application message. |
| `Enabled` | true. Controls native dragging. |

## Interaction

Drag from the grip or another part of the row outside interactive children.
CUI moves the surrounding rows as you drag. On drop, CUI proposes the new
permutation if the position changed. Apply it in the update and keep the rows
in their canonical order.

CUI draws the grip and centers the direct row children vertically. Each
interactive child keeps its own input area. Set `Enabled` to `false` to
disable dragging.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_reorder_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/ReorderableLists.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/Widgets/Tasks.fpas)
- [Gallery drag test](../../../../tests/Fixtures/gui-host/GalleryTableContract.fpas)
- [Virtual list](virtual-list.md)
- [Data display](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [ReorderableList.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/ReorderableList.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.ReorderableList.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.ReorderableList.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.ReorderableList.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.ReorderableList.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [reorderable_list.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/reorderable_list.c)
  - [reorderable_list.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/reorderable_list.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Display/`
  - [CuiReorderableListTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiReorderableListTests.cs)
