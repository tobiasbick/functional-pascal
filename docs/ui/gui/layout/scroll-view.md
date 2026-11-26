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
public function View(Cards: array of E.Element of string): E.Element of string;
begin
  return E.ScrollView of string(
    'previews',
    E.Row of string(new L.Flow(Gap := L.Spacing.Normal, Clip := false), Cards),
    new L.ScrollOptions(
      FadeEdges := true, Horizontal := true, Height := L.Extent.Fixed(L.Measure.Logical(150))
    )
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `ShowScrollbar` | `true`. Set `false` to hide the scrollbar and keep wheel input. |
| `Horizontal` | `false`. Set `true` to scroll on both axes. |
| `Width / Height` | Fill. Supply a fixed extent or a percentage to size the viewport. Content width fits the children. Content height uses Fill. |
| `Padding / Gap` | Zero. Supply an inset on every edge and a gap between direct children. The array overload places its children vertically. |
| `FadeEdges` | `false`. Set `true` to fade edges with more content beyond them. |
| `FadeColor` | CUI uses the nearest opaque container fill, or the theme background when no opaque fill exists. |

The host owns the scroll position when the view omits `Position` and
`OnScroll`. The model owns the vertical position when the view supplies
both. GUI scroll proposals use fractional logical units.
See the [scroll position contract](../../layout/scroll-position.md).

## Reveal a child

Use `Layout.ScrollPosition.Reveal('selected-row')` to bring a named child
into view. The GUI host uses the viewport and child bounds from the preceding
layout. The host waits until those bounds exist.
A fully visible child keeps its position. A partially visible child moves
only far enough to show the clipped edge. A child taller than the viewport
aligns its top with the viewport.
The target must belong to the viewport. The host skips targets inside a
nested scroll viewport.

`OnScroll` reports a changed offset. Store the returned `Position` to resume
free scrolling after the reveal.

## Interaction

Constrain the viewport along the scroll axis so content can overflow.
In the horizontal example, use horizontal wheel input or hold Shift while
turning a vertical scroll wheel. Drag the visible scrollbar to scroll
vertically.
For anchored navigation, let the host own the offset and put the outline
in the same CUI scope.

## See also

- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Scroll anchor](../reader/scroll-anchor.md)
- [Table of contents](../reader/table-of-contents.md)
- [GUI layout](README.md)
- [Terminal example](../../tui/layout/scroll-view.md)

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Scroll view presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [Scroll options contract tests](../../../../tests/Fixtures/gui-host/ScrollOptionsContract.fpas)
