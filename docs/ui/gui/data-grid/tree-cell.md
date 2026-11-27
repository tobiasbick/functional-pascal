# Tree cell

The gallery uses tree cells in the Name column of its ordinary source
table. The cells indent names and let the user expand folders.
Available in **GUI**.

![Indented names in the gallery's source table](images/tree-cells.png)

## Example

```pascal
unit Docs.Ui.DataGrid.TreeCell;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.Tables as T;
uses Fpas.Ui.Widgets.Collections.TreeTables as W;

// Displays a source folder inside an ordinary table.
// ## Parameters
// - `Open`: Whether the folder's child should be visible.
// ## Returns
// An expandable table with the proposed expansion as its message.
public function View(Open: bool): E.Element of bool;
begin
  const Folder: T.Row of bool := new T.Row of bool('src', [
    (T.Cell of bool).Content([W.TreeCell of bool('src-name', 'src', 0, Open,
      function(Expanded: bool): bool => Expanded)]),
    (T.Cell of bool).Text('8 KB')
  ]);
  const Source: T.Row of bool := new T.Row of bool('main', [
    (T.Cell of bool).Content([W.TreeCell of bool('main-name', 'main.c', 1)]),
    (T.Cell of bool).Text('8 KB')
  ]);
  const Rows: array of T.Row of bool := if Open then [Folder, Source] else [Folder] end if;
  return T.Table of bool('source-files', [new T.Column('Name'), new T.Column('Size')], Rows);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | A stable cell ID. Cells with the same label still need distinct IDs. |
| `Label` | Literal text. |
| `Depth` | A nonnegative indentation level. |
| `Open` | Null for a leaf. Pass `true` for an expanded branch or `false` for a collapsed branch. |
| `OnChange` | Null for a read-only branch. Pass a mapper to receive the proposed Boolean after a click. |
| `Icon` | Empty. Supply a CUI glyph to show an icon beside the name. |

## Interaction

Place the tree cell inside `Tables.Cell.Content` or `DataGrids.Cell.Content`.
Build the visible rows from the model. When a branch sends a message, store
the proposed expansion in the model. Omit its descendants when it collapses.

Set `Depth` to zero for a root. Each level adds 18 logical units of
indentation inside the column. A leaf reserves the same space as a chevron.
The chevron accepts pointer input. Use a [tree table](tree-table.md) for
keyboard navigation between branches. In a GUI snapshot, a branch reports
the chevron bounds and a leaf reports the cell content bounds.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_tree_table_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/TreeTables.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [Table](../data-display/table.md)
- [Tree table](tree-table.md)
- [Data grid](README.md)
