# Panels and weighted layout

Compose a panel from a border, a semantic surface role and a size rule.
The gallery caps the sidebar at 24 columns and gives the remaining space
to the preview.

![Two rounded, differently sized panels with a one-cell gap](../../../../examples/ui/tui/gallery/panels.png)

## Example

```pascal
unit Docs.Tui.Widgets.Panels;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Styles as S;

// Builds the panels and weighted layout example.
// ## Parameters
// - `Content`: Panel body.
// - `Weight`: Share of the available width.
// ## Returns
// The resulting view.
public function View(Content: E.Element of string; Weight: float): E.Element of string;
requires Weight > 0.0F;
begin
  return E.Size of string(E.Styled of string(E.Border of string(Content), S.Role.Panel),
    new L.Flex(Basis := L.Extent.Fill, Grow := Weight));
end function;

end unit;
```

## Border and inner layout

`Elements.Border` reserves one cell on each edge. Set its `Title` to write a
label on the top edge. Set padding on a row or column inside the border to
add space between the content and the frame. `Elements.Styled` with
`Styles.Role.Panel` applies the theme's panel colors.

The panel adds no focus stop. Its controls participate in the enclosing
focus scope.

## Size a panel

Place two fill panels in a row with weights one and two to share the
available width proportionally. Use a fixed width for a sidebar to give
the remaining width to a fill panel. Use `Flow.Gap` with
`Spacing.Fixed(Measure.Cells(1))` for one cell between the panels.

## See also

- [Layout](README.md)
- [Rows and columns](rows-and-columns.md)
- [Measures, extents and size rules](../../layout/sizing.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Arrangement.fpas](../../../../src/Fpas.Ui.Tui/Internal/Layout/Arrangement.fpas)
- [Drawing.fpas](../../../../src/Fpas.Ui.Tui/Internal/Rendering/Drawing.fpas)
