# Scroll view

Keep a long list or body inside a bounded area. The left gallery panel scrolls independently of the right preview. When the list overflows, a heavier vertical line (`┃`) marks the visible portion on a thin track (`│`).

![Long sidebar clipped by its panel with the scrollbar thumb along its right edge](../../../../examples/ui/tui/gallery/panels.png)

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

The size in this example applies inside a column. The wheel scrolls the viewport beneath the pointer. A sidebar reveals the row reached by keyboard navigation. Use the controlled overload when the model needs to remember the vertical position; keep the scroll ID stable.

## See also

- [Layout](README.md)
- [Shared API](../../widgets.md#scrolling)
- [GUI scroll view](../../gui/layout/scroll-view.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [ScrollNavigation.fpas](../../../../src/Fpas.Ui.Tui/Internal/Hosting/ScrollNavigation.fpas)
- [ScrollViews.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/ScrollViews.fpas)
