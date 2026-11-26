# Column

`Elements.Column` lays out children vertically. The GUI host uses a native
CUI column. The TUI host also supports columns.

## Example

```pascal
unit Docs.Ui.Layout.Column;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Theming.Themes as T;

// Builds a column with a themed surface and its own scrollbar.
// ## Returns
// A fixed-height column with native clipping and vertical scrolling.
public function View(): E.Element of string;
begin
  return E.Size of string(
    E.Column of string(
      new L.Flow(
        Id := 'activity',
        PaddingX := L.Spacing.Loose, PaddingY := L.Spacing.Normal, Gap := L.Spacing.Tight,
        BackgroundTheme := T.ColorToken.SurfaceRaised, Border := new L.Border(),
        CornerRadius := 10.0F, Corners := new L.CornerRadii(TopLeft := 0.0F),
        Elevation := L.Elevation.Resting, Scrolling := L.ScrollMode.Vertical
      ),
      E.Text of string('Recent activity'),
      E.Text of string('Library scan complete'),
      E.Text of string('Playlist refreshed'),
      E.Text of string('New albums indexed'),
      E.Text of string('Artwork cache updated')
    ),
    new L.Flex(Height := new L.AxisSize(Basis := L.Extent.Fixed(L.Measure.Logical(100))))
  );
end function;

end unit;
```

## Alignment and scrolling

`Align` places children horizontally. The default stretches children across
the available width. Controls with an intrinsic width keep that width.
`Justify` distributes free space vertically.

Constrain the height to create vertical overflow for a scrolling column.
Use a scroll view if the model owns the scroll position.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Overview example](../../../../examples/ui/gui/gallery/Pages/Overview/View.fpas)
- [Container options](container-options.md)
- [Spacing and padding](spacing.md)
- [Sizing](../../layout/sizing.md)
- [Scroll view](scroll-view.md)
- [TUI layout](../../tui/layout/README.md)
- [GUI layout](README.md)

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [GUI flow presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [Native pixel parity tests](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
- [FPAS layout contract](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
