# Row

`Elements.Row` lays out children horizontally in GUI and TUI. GUI uses a native
CUI row and can wrap children onto additional lines.

## Example

```pascal
unit Docs.Ui.Layout.Row;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;

// Arranges filters in a native wrapping row.
// ## Returns
// A row that wraps at the width offered by its parent.
public function View(): E.Element of string;
begin
  return E.Size of string(
    E.Row of string(
      new L.Flow(
        Wrap := true, Align := L.Alignment.Center, Justify := L.Justification.Between,
        Gap := L.Spacing.Fixed(L.Measure.Points(8.5F)),
        ReadingDirection := L.Direction.Auto
      ),
      E.Text of string('Genre'),
      E.Text of string('Release year'),
      E.Text of string('Rating')
    ),
    new L.Flex(Width := new L.AxisSize(Basis := L.Extent.Fill))
  );
end function;

end unit;
```

The default alignment centers children vertically. `Wrap` uses the row's
available inner width to break lines. `Align` positions children vertically
within each line, `Justify` distributes free space horizontally per line, and
`Gap` separates both children and lines.

FPAS clips non-wrapping rows by default. Wrapping rows can paint past their
bounds unless `Clip` is enabled. Scrolling always clips to the viewport.

Rows and columns share sizing, padding, clipping, styling, identity and
scrolling through `Elements.Size` and `Layout.Flow`. The [column
reference](column.md) lists these options. Reading direction can be inherited,
explicit or automatic. `Auto` selects direction from the first strong
character in descendant text. See [Reading direction](reading-direction.md).

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [LayoutContract.fpas](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [Native layout comparison](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
