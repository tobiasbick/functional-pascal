# Table of contents

Reader pairs an outline with a scrolling article. The outline indents nested
headings and marks the current section.
Available in **GUI**.

![Reader outline with Introduction active](images/table-of-contents.png)

## Example

```pascal
unit Docs.Ui.Reader.TableOfContents;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Navigation.TableOfContents as W;

// Links an outline to the article viewport.
// ## Returns
// The outline, with the active anchor ID as a message.
public function View(): E.Element of string;
begin
  return W.TableOfContents of string('outline', 'article', [
    new W.Item('Introduction', 'intro'),
    new W.Item('Installation', 'install'),
    new W.Item('With CMake', 'cmake', 1)
  ], function(Anchor: string): string => Anchor);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `ScrollId` | The ID of an uncontrolled `ScrollView` in the same CUI scope. Each scroll view accepts one outline. |
| `Items` | One to 64 entries with nonempty titles. List them in the order of their anchors in the scroll view. |
| `Item.Title` | Literal text in the outline row. |
| `Item.Anchor` | A distinct anchor ID for each entry. |
| `Item.Level` | Zero. CUI adds 12 logical units of indentation per level, up to level 4. |
| `OnActiveChange` | Null. Supply a callback to observe changes after the initial frame. |
| `Enabled` | True. Allows focus and activation. The marker continues to track scrolling when disabled. |

## Interaction

Click an entry or press Enter or Space while it has focus. CUI scrolls the
anchor toward an inset 12 logical units below the top of the viewport.
The available scroll range limits the destination near either end.

Use `ItemId(Id, Index)` to request focus for an entry by its zero-based index.
The active marker follows the last heading that reaches the upper 30 percent
of the viewport. The first entry is the default before the first layout.
The final entry is active at the end of a scrollable article. The rail shows
scroll progress.
The GUI snapshot exposes the active anchor ID as the outline's value.

Place each marker inside the target scroll view. Keep the outline and its
target in the same CUI scope. A viewport inside a dialog cannot be targeted
by an outline outside the dialog.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_toc)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/TableOfContents.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Reader/View.fpas)
- [Scroll anchor](scroll-anchor.md)
- [Scroll view](../layout/scroll-view.md)
- [Chapter index](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [TableOfContents.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/TableOfContents.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.TableOfContents.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.TableOfContents.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.TableOfContents.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.TableOfContents.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [table_of_contents.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/table_of_contents.c)
  - [table_of_contents.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/table_of_contents.h)
