# Scroll view

Keep a long list or body inside a bounded area. The gallery preview scrolls independently of the sidebar. When the content overflows, a heavier vertical line (`┃`) marks the visible portion on a thin track (`│`).

![Preview scrolled to the segmented controls with its scrollbar focused](../../../../examples/ui/tui/gallery/scroll-view.png)

## Example

```pascal
unit Docs.Tui.Widgets.ScrollView;

uses Fpas.Ui.Elements as E, Fpas.Ui.Layout as L;

// Builds the scroll view example.
// ## Parameters
// - `Items`: Rows to display.
// ## Returns
// The resulting view.
public function View(Items: array of E.Element<string>): E.Element<string>;
begin
  return E.Size<string>(E.ScrollView<string>('tracks-scroll', E.Column<string>(Items)),
    new L.Flex(Basis := L.Extent.Fixed(L.Measure.Cells(8))));
end function;

end unit;
```

## Interaction

The size in this example applies inside a column. The wheel scrolls the viewport
beneath the pointer. A sidebar reveals the row reached by keyboard navigation.
Use the controlled overload to let users page through passive content. Store
the reported position in the model and keep the scroll ID stable. When it
overflows, the viewport takes one tab stop and its scrollbar highlights focus.
Up, Down, Page Up, Page Down, Home and End scroll it while it has focus.

## See also

- [Layout](README.md)
- [Shared API](../../widgets.md#scrolling)
- [GUI scroll view](../../gui/layout/scroll-view.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [ScrollNavigation.fpas](../../../../src/Fpas.Ui.Tui/Internal/Hosting/ScrollNavigation.fpas)
- [ScrollViews.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/ScrollViews.fpas)
