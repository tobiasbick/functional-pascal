# Reading direction and spacing

Layout shows the same Start and End controls in left-to-right and
right-to-left rows.
Available in **GUI**.

![Left-to-right and right-to-left arrangement in the FPAS gallery](images/reading-direction.png)

## Example

```pascal
unit Docs.Ui.Layout.ReadingDirection;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;

// Places the two labels on opposite sides of a mirrored row.
// ## Returns
// A row with explicit leading inset, gap and spacer.
public function View(): E.Element of string;
begin
  return E.Row of string(
    new L.Flow(
      ReadingDirection := L.Direction.RightToLeft,
      Padding := L.Spacing.Fixed(L.Measure.Logical(12)),
      PaddingStart := L.Spacing.Fixed(L.Measure.Logical(24))
    ),
    E.Text of string('Start'),
    E.Gap of string(L.Measure.Logical(12)),
    E.Spacer of string(),
    E.Text of string('End')
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Desktop.Options.ReadingDirection` | Initial direction of the window. `Inherit` defaults to left to right. |
| `ReadingDirection` | Inherit. `LeftToRight`, `RightToLeft` and `Auto` require the GUI host. |
| `PaddingStart / PaddingEnd` | `null`. The resolved direction maps each logical edge to the physical left or right edge. |

## Usage

Use start and end edges to mirror a row with the reading direction.
The element order in the view and the labels stay the same.
`Auto` uses CUI's first strong directional character in descendant text.
CUI skips subtrees with an explicit direction. If it finds only neutral
characters, the container inherits its parent's direction.

Set `Desktop.Options.ReadingDirection` to configure the whole window.
`Layout.Flow.ReadingDirection` overrides a subtree and `Texts.Options` can
override one text element. Text Start and End follow the resolved direction.
Left and Right select physical edges. Native floating layers inherit from
their opening container, and native horizontal scrolling follows the area’s
direction. Canvas coordinates stay physical. CUI’s filled text fields still
edit left to right.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Window options](../host.md#application-lifecycle)
- [Native text options](../text-and-display/text.md)
- [Spacing and padding](spacing.md)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Layered stack](stack.md)
- [Row](row.md)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Desktop.fpas](../../../../src/Fpas.Ui.Gui/Desktop.fpas)
- [Texts.fpas](../../../../src/Fpas.Ui/Widgets/Display/Texts.fpas)
- [Layout direction conversion](../../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
