# Panels and weighted layout

Compose a panel from a border, a semantic surface role and a size rule. The gallery splits its available columns in a 1:2 ratio.

![Two rounded, differently sized panels with a one-cell gap](../../../../examples/ui/tui/gallery/panels.png)

## Example

```pascal
unit Docs.Tui.Widgets.Panels;

uses Fpas.Ui.Elements as E, Fpas.Ui.Layout as L, Fpas.Ui.Styles as S;

// Builds the panels and weighted layout example.
// ## Parameters
// - `Content`: Panel body.
// - `Weight`: Share of the available width.
// ## Returns
// The resulting view.
public function View(Content: E.Element<string>; Weight: float): E.Element<string>;
requires Weight > 0.0F;
begin
  return E.Size<string>(E.Styled<string>(E.Border<string>(Content), S.Role.Panel),
    new L.Flex(Basis := L.Extent.Fill, Grow := Weight));
end function;

end unit;
```

## Interaction

Place two panels in a row with weights 1 and 2. The gap is removed before the remaining width is shared. Use `L.Measure.Cells(1)` for a one-column terminal gap. Resize the terminal to see the panels share the new width. The panel is a container and adds no focus stop.

## See also

- [Layout](README.md)
- [Shared API](../../widgets.md#layout-and-style)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)
