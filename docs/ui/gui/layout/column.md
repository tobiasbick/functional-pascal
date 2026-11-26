# Column

`Elements.Column` lays out children vertically. Rows and columns are available
in GUI and TUI. The additional options below use the native CUI box in GUI.

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

| Option | Behavior |
| --- | --- |
| `E.Size` | Constrain width and height independently. Both axes support content, percentage, fixed and weighted fill with optional bounds. |
| `Align / Justify` | Align children horizontally and distribute them vertically. |
| `Gap / Padding` | Use semantic or explicit logical distances, including fractions through `Measure.Points`. Edge insets take precedence over axis insets and uniform padding. |
| `ReadingDirection` | Inherit, select left-to-right or right-to-left, or use `Auto` to detect the direction from the text. Start and end insets follow the direction. |
| `Background / BackgroundTheme` | Choose an optional RGBA fill or a semantic theme color. |
| `Border` | Set the width and an optional RGBA or theme color. Omitting the color uses the active theme border. |
| `CornerRadius / Corners` | Set a common radius and override physical corners individually, including with zero. |
| `Elevation / Opacity` | Select the native theme shadow and the paint opacity of the column and its children. |
| `Clip` | Ordinary FPAS columns clip by default. Set false to allow paint past their bounds. Scrolling always clips to the viewport. |
| `Scrolling / Id` | Native scrolling supports vertical or both axes. Supply a stable ID to retain the native position across changes within the same ID scope. Omitted IDs use CUI's automatic scroll identity. A named column is also visible in GUI frame inspection. |

Use [ScrollView](scroll-view.md) when the model owns the scroll position or the
application needs scroll events. Native row wrapping and grid columns have
their own options through [Row](row.md) and [Grid](../feedback/grid.md).

Nested `Elements.Size` calls can constrain one axis at a time. A width-only
constraint preserves any height bounds already set on the box.

## Source files

- [Layout types](../../../../src/Fpas.Ui/Layout.fpas)
- [GUI flow presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [Native pixel parity tests](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
- [FPAS layout contract](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
