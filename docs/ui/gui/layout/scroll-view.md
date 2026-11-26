# Scroll view

Layout shows a strip of cards inside a horizontal viewport with an edge fade.
Horizontal scrolling and edge fades require the GUI host.
Available in **GUI and TUI**.

![Horizontal card strip in the FPAS gallery](images/scroll-view.png)

## Example

```pascal
unit Docs.Ui.Layout.ScrollView;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;

// Shows cards in a bounded horizontal viewport.
// ## Parameters
// - `Cards`: The row contents in display order.
// ## Returns
// A scroll view that owns its offset.
public function View(Cards: array of E.Element<string>): E.Element<string>;
begin
  return E.Size<string>(
    E.ScrollView<string>('previews',
      E.Row<string>(new L.Flow(Gap := L.Spacing.Normal, Clip := false), Cards),
      FadeEdges := true, Horizontal := true),
    new L.Flex(Height := new L.AxisSize(Basis := L.Extent.Fixed(L.Measure.Logical(150))))
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `ShowScrollbar / FadeEdges / Horizontal` | true / false / false. Enable Horizontal to allow scrolling on both axes. |
| `Position / OnScroll` | Use the controlled overload to store the vertical position in the model and receive scroll proposals. |

## Interaction

Constrain the viewport along the scroll axis so overflow can scroll. In
the horizontal example, use horizontal wheel input or hold Shift while
turning a vertical scroll wheel. Drag the scrollbar to scroll vertically
when content overflows that axis.
For anchored navigation, let the host own the offset and put the outline
in the same CUI scope.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Elements.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Scroll anchor](../reader/scroll-anchor.md)
- [Chapter index](README.md)
- [Terminal example](../../tui/layout/scroll-view.md)
