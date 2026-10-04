# `Std.Tui` cells and styles

`TuiColor` has distinct constructors for each representation. `Cells.TuiColorFromCrt` accepts
`0..15`; `Cells.TuiColorFromAnsi256` and every `Cells.TuiColorFromRgb` channel accept `0..255`.

```pascal
uses Std.Tui as Tui;
uses Std.Tui.Cells as Cells;

const Foreground: Tui.TuiColor := Cells.TuiColorFromCrt(14);
const Background: Tui.TuiColor := Cells.TuiColorFromRgb(10, 20, 30);
const Style: Tui.TuiStyle := Cells.TuiStyleFromColors(Foreground, Background);
const Cell: Tui.TuiCell := Cells.TuiCellCreate('X', Tui.TuiStyleRole.Focused);
const TruecolorCell: Tui.TuiCell := Cells.TuiCellStyled('▓', Style);
```

`Cells.TuiStyleCreate` additionally accepts `Bold`, `Dim`, `Underline`, and
`Inverse` flags. `Cells.TuiCellCreate` requires exactly one non-zero-width extended
grapheme cluster and stores its terminal column width (`1` or `2`).
`Cells.TuiCellWidth(Cell)` returns that stored value.

`Cells.TuiCellCreate` stores a semantic `TuiStyleRole`; palette lookup supplies
concrete colors. `Cells.TuiCellStyled` stores a concrete style that bypasses palette
lookup. This is useful inside `TuiCellGrid` for plots and images whose colors
are data rather than theme roles. Continuation cells for wide glyphs remain
private surface state and are not part of the public cell value.

`Rendering.TuiWorkingSurfaceResize(Surface, Size)` replaces the mutable grid with a blank grid of
the requested size while preserving the surface handle. Existing snapshots stay
immutable, and resizing one surface does not affect other surfaces.

## `TuiPalette`

`Cells.TuiPaletteDefault()` provides the standard semantic colors. `Cells.TuiPaletteForRole` resolves
one style and `Cells.TuiPaletteWithRole` returns a copy with one replacement, leaving the
original palette unchanged.

The RGB default palette uses a dark terminal background, restrained borders,
and blue selection accents. General roles (`Normal`, `Focused`, `Frame`, and
`Title`) style the desktop, panels, and overlays. Menus use `MenuNormal`,
`MenuDisabled`, `MenuShortcut`, `MenuSelected`, and
`MenuSelectedShortcut`; status lines use
`StatusNormal`, `StatusDisabled`, `StatusShortcut`, and `StatusSelected`. The
menu bar and status line paint their complete row with their normal role, so
their chrome remains distinct from the desktop.

Buttons use `ButtonNormal`, `ButtonDefault`, `ButtonSelected`,
`ButtonDisabled`, `ButtonShortcut`, `ButtonDefaultShortcut`,
and `ButtonSelectedShortcut`. The corresponding styles are grouped in
`TuiPalette.Buttons` as `TuiButtonPalette`. `Cells.TuiButtonPaletteWithRole` returns
a copy with one button role replaced, while `Cells.TuiButtonPaletteForRole` resolves one button role.

`Rule`, `GaugeTrack`, and `GaugeFill` style the dashboard primitives.

One-line inputs use `InputNormal`, `InputFocused`, `InputHint`, `InputCursor`,
and `InputScroll`, grouped in `TuiPalette.Inputs` as `TuiInputPalette`. Themes
should keep these roles on a common field background while changing foreground
or inverse cursor colors. `Cells.TuiInputPaletteWithRole` returns a copy with one
input role replaced, while `Cells.TuiInputPaletteForRole` resolves one input role.

```pascal
uses Std.Tui as Tui;
uses Std.Tui.Cells as Cells;

const Palette: Tui.TuiPalette := Cells.TuiPaletteDefault();
const Warning: Tui.TuiStyle := Cells.TuiPaletteForRole(Palette, Tui.TuiStyleRole.Warning);
const Custom: Tui.TuiStyle := Cells.TuiStyleFromColors(Cells.TuiColorFromRgb(255, 128, 0), Cells.TuiColorFromCrt(0));
const Updated: Tui.TuiPalette := Cells.TuiPaletteWithRole(Palette, Tui.TuiStyleRole.Accent, Custom);
```

A palette is ordinary public FPAS data. Applications start from
`Cells.TuiPaletteDefault()` and replace the roles they need with `Cells.TuiPaletteWithRole`. This is
the theme extension boundary; no theme registry or fixed set of color names is
required.

Use `Runtime.TuiApplicationOpenForTestWithPalette` or `Runtime.TuiApplicationRunWithPalette` to select the initial palette.
An Update function can switch it immediately:

```pascal
uses Std.Tui.Runtime as Runtime;
Runtime.TuiCmdOutputSetPalette(Cmd, MyPalette);
```

The next interactive frame is fully recolored even when its glyphs and
semantic roles did not change. `Runtime.TuiApplicationPalette(App)` exposes the active palette for
headless assertions.

## See also

- [`Std.Tui`](README.md)
- [Layout](layout.md)
- [Application](application.md)
