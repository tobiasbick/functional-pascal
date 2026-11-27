# Tree

Use a tree for nested nodes such as files and folders. Compose each group with Branch and each final item with Leaf. Available in **GUI** and [TUI](../../tui/data-display/tree.md).

![Tree in the FPAS gallery](images/tree.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.Tree;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.Trees as W;

// Changes proposed by the source tree.
public type Msg = variant
  // Change the expansion of the source branch.
  SetSource(Open: bool);
  // Select a file by name.
  SelectFile(Name: string);
end variant;

// Builds the tree example.
// ## Parameters
// - `Open`: Expansion flag from the application model.
// ## Returns
// The widget for the application view.
public function View(Open: bool): E.Element of Msg;
begin
  return W.Branch of Msg('tree-src', 'src', Open, [
    W.Leaf of Msg('file-main', 'main.c', function(): Msg => Msg.SelectFile('main.c')),
    W.Leaf of Msg('file-table', 'table.c', function(): Msg => Msg.SelectFile('table.c'))
  ], Msg.SetSource);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Open / OnChange` | Pass the current flag to Branch and handle the next flag in the model. |
| `OnActivate` | Null. Supply a message factory for a leaf or for a GUI branch. |
| `Icon` | Null uses the native file or folder icon. An empty string draws no glyph. |
| `Selected` | False. True applies CUI's selected background and text weight. |
| `Enabled` | True. False disables row input. |

## Interaction

Click a branch or press Enter or Space to toggle it. Right opens a focused
branch and Left closes it. `OnChange` proposes the new expansion state. CUI
draws the chevron and indents the children.

Click a row or press Enter or Space to invoke its `OnActivate` factory. If both
callbacks apply to a branch, the GUI dispatches the expansion message first.
Give each row a stable ID independent of its caption.

In the gallery, the four component files can change the selection. Expand
the assets branch to reveal the logo. The logo and `main.c` keep the current
file selection when clicked.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_tree_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/Trees.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/Widgets/TreeAndList.fpas)
- [List item](list-item.md)
- [Data display](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [Tree.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/Tree.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.Tree.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.Tree.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.Tree.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.Tree.Types.cs)
  - [CuiWindow.Tree.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.Tree.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [tree.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/tree.c)
  - [tree.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/tree.h)
- [CuiTreeTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiTreeTests.cs)

- [Shared source tree sample](../../../../examples/ui/gui/gallery/Shared/SourceTree.fpas)
