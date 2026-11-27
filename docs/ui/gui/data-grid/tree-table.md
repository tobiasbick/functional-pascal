# Tree table

Use a tree table when nested rows need columns for values such as file count
and size. The widget realizes the visible rows. The Files gallery uses
dragging to move rows in a sample project. Available in **GUI**.

![Virtual tree table with file totals](images/tree-table.png)

## Example

```pascal
unit Docs.Ui.DataGrid.TreeTable;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Collections.DataGrids as D;
uses Fpas.Ui.Widgets.Collections.Tables as T;
uses Fpas.Ui.Widgets.Collections.TreeTables as W;

// Builds the file summary tree.
// ## Parameters
// - `Nodes`: Nodes in sibling order, with parents before their children.
// - `Cursor`: Current node index or minus one.
// ## Returns
// The tree with each proposed change as an application message.
public function View(Nodes: array of W.Node; Cursor: int): E.Element of W.Change;
begin
  return W.TreeTable of W.Change('project-tree',
    [new D.Column('Name', 320.0F), new D.Column('Size', 100.0F, T.Alignment.Right)],
    Nodes, Cursor, null, function(Change: W.Change): W.Change => Change,
    new W.Options(Height := L.Extent.Fixed(L.Measure.Logical(240)), Guides := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Columns / Node.Cells` | Put Name first. Each node supplies one text value for every remaining column. |
| `Node.Parent` | Minus one for a root. For a child, use the index of a preceding parent. |
| `Node.Icon` | An empty string omits the glyph. |
| `Node.HasChildren / Open / Selected / Loading` | All false. A branch can advertise children before they are loaded. |
| `Node.TypeAheadLabel` | Null uses the row label for type-ahead and the drag preview. |
| `Cursor / Order` | Use minus one for no initial cursor and null to disable sortable headers. |
| `Width / Height / RowHeight` | Fill / Fill / 32 logical units. Zero also selects the default row height. |
| `Guides / Enabled` | False / true. |
| `TypeAhead` | True. Set false to omit the native label callback for type-ahead and the drag preview. |
| `Draggable` | False. Set true to receive node move proposals. |
| `CanDrop` | Null. An optional predicate accepts a target index and placement. |

## Column and row layout

The first column contains the label, chevron, and indentation for each node.
`Guides` draws lines that show the depth. CUI derives the visible rows from
the open branches. Each visible row uses `RowHeight`.

Columns use the data grid's initial widths, resizing, priority hiding, and
direction-aware alignment. See [data grid layout](data-grid.md#column-and-viewport-layout).

## Interaction

Arrow keys navigate and expand or collapse branches. Enter activates the
current node. Double-click a row to activate it. Store the proposed nodes and
cursor. `Change.Expanded` and `Change.Collapsed` identify the affected branch.
If `Change.SortChanged` is true, sort each sibling group using `Change.Order`.
For each index in `Change.Load`, supply the children and clear the parent's
loading flag after the load finishes.

Use a [tree cell](tree-cell.md) inside an ordinary table when the
application builds the visible rows itself.

## Move nodes

When dragging starts on a selected row, the drag carries the selected
subtrees in tree order. Dropping onto a valid target returns `Change.Drop`.
`Drop.Placement` specifies before the target, inside it as its last children,
or after it.

Call `Fpas.Ui.Gui.TreeTables.Move(Change.Nodes, Drop)` to apply the move to
the model. Read `MoveResult.IndexMap[OldIndex]` to update the cursor and other
stored indices. Open the destination branch if the application must show its
new children. The application owns any corresponding file operations.

## Context menu

Pass `Menu` to give each node a native context menu. A right click opens the
menu for the row at the pointer. Shift+F10 opens it for the cursor row. If
that row is outside the selection, the table first proposes it as the new
selection. The menu builder receives the node index. Use the standard menu
entries for commands, labels and separators. [Menu entries](../actions/dropdown-menu.md#options-and-defaults)

Use Up, Down and Enter to choose a command. Escape closes the menu. The menu
also closes if its node disappears or the table stops accepting input. For a
copy command, return `Clipboard.WriteText` from the application update.
[Clipboard commands](../host.md#clipboard)

If the pointer moves between rows before CUI processes a right press, the
menu can open for the destination row. [CUI-125](../../../future/cui-upstream-bugs.md#cui-125)

```pascal
unit Docs.Ui.DataGrid.TreeTableMenu;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.DataGrids as D;
uses Fpas.Ui.Widgets.Collections.TreeTables as W;
uses Fpas.Ui.Widgets.Overlays.Menus as M;

// Messages of the file tree.
public type Msg = variant
  // Store the proposed tree state.
  Changed(Value: W.Change);
  // Delete one node.
  Delete(Node: int);
end variant;

// Builds the entries for one node.
// ## Parameters
// - `Node`: Model index of the node the menu is open for.
// ## Returns
// The commands for that node.
function Commands(Node: int): array of M.Entry of Msg;
begin
  return [(M.Entry of Msg).Action(new M.Item of Msg('delete', 'Delete', Msg.Delete(Node), Danger := true))];
end function;

// Builds a file tree whose rows have a context menu.
// ## Parameters
// - `Nodes`: Nodes in sibling order, with parents before their children.
// ## Returns
// The tree table.
public function View(Nodes: array of W.Node): E.Element of Msg;
begin
  return W.TreeTable of Msg('files', [new D.Column('Name', 320.0F)], Nodes, -1, null,
    function(Change: W.Change): Msg => Msg.Changed(Change),
    Menu := function(Node: int): array of M.Entry of Msg => Commands(Node));
end function;

end unit;
```

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_tree_table_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/TreeTables.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [Files gallery](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)
- [Gallery update function](../../../../examples/ui/gui/gallery/Pages/DataGrid/Update.fpas)
- [Data grid](README.md)

## Source files

- [TreeTables.fpas](../../../../src/Fpas.Ui.Gui/TreeTables.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [TreeTable.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/TreeTable.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.TreeTable.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.TreeTable.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.TreeTable.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.TreeTable.Types.cs)
  - [CuiWindow.TreeTable.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.TreeTable.cs)
  - [CuiWindow.TreeTableMenu.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.TreeTableMenu.cs)
  - [CuiTreeMoves.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiTreeMoves.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [tree_table.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/tree_table.c)
  - [tree_table.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/tree_table.h)
  - [tree_table_menu.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/tree_table_menu.c)
- `tests/Fixtures/gui-host/`
  - [TreeTableMenuContract.fpas](../../../../tests/Fixtures/gui-host/TreeTableMenuContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Display/`
  - [CuiTreeMovesTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiTreeMovesTests.cs)
- `tests/Fpas.Ui.Gui.Cui.Tests/Overlays/`
  - [CuiTreeTableMenuTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiTreeTableMenuTests.cs)
