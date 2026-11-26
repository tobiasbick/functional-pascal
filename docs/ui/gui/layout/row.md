# Row

`Elements.Row` lays out children horizontally. The GUI host uses a native
CUI row and can wrap children onto additional lines. The TUI host also
supports rows.

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

## Alignment and wrapping

`Align` places children vertically. The default centers children.
`Justify` distributes free space horizontally.

With `Wrap := true`, CUI uses the inner width to decide where to break each
line. A growing child's minimum width contributes to that decision.
CUI then distributes the available width among the children on each line.
`Align` places children within the height of their line.
`Gap` separates adjacent children and lines.

## See also

- [Container options](container-options.md)
- [Spacing and padding](spacing.md)
- [Reading direction](reading-direction.md)
- [Responsive layout](../../layout/responsive.md)
- [TUI layout](../../tui/layout/README.md)
- [GUI layout](README.md)

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [LayoutContract.fpas](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [Native layout comparison](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
