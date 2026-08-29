# `Std.Tui` cells and styles

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts
`0..15`; `FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal
var Foreground: TuiColor := TuiColor.FromCrt(14);
var Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
var Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
var Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
var TruecolorCell: TuiCell := TuiCell.Styled('▓', Style);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and
`Inverse` flags. `TuiCell.Create` requires exactly one non-zero-width extended
grapheme cluster and stores its terminal column width (`1` or `2`).
`Width()` returns that stored value.

`TuiCell.Create` stores a semantic `TuiStyleRole`; palette lookup supplies
concrete colors. `TuiCell.Styled` stores a concrete style that bypasses palette
lookup. This is useful inside `TuiCellGrid` for plots and images whose colors
are data rather than theme roles. Continuation cells for wide glyphs remain
private surface state and are not part of the public cell value.

`TuiWorkingSurface.Resize(Size)` replaces the mutable grid with a blank grid of
the requested size while preserving the surface handle. Existing snapshots stay
immutable, and resizing one surface does not affect other surfaces.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves
one style and `WithRole` returns a copy with one replacement, leaving the
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
`TuiPalette.Buttons` as `TuiButtonPalette`. `TuiButtonPalette.WithRole` returns
a copy with one button role replaced, while `ForRole` resolves one button role.

`Rule`, `GaugeTrack`, and `GaugeFill` style the dashboard primitives.

One-line inputs use `InputNormal`, `InputFocused`, `InputHint`, `InputCursor`,
and `InputScroll`, grouped in `TuiPalette.Inputs` as `TuiInputPalette`. Themes
should keep these roles on a common field background while changing foreground
or inverse cursor colors. `TuiInputPalette.WithRole` returns a copy with one
input role replaced, while `ForRole` resolves one input role.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

A palette is ordinary public FPAS data. Applications start from
`TuiPalette.Default()` and replace the roles they need with `WithRole`. This is
the theme extension boundary; no theme registry or fixed set of color names is
required.

Use `OpenForTestWithPalette` or `RunWithPalette` to select the initial palette.
An Update function can switch it immediately:

```pascal
Cmd.SetPalette(MyPalette);
```

The next interactive frame is fully recolored even when its glyphs and
semantic roles did not change. `App.Palette()` exposes the active palette for
headless assertions.

## See also

- [`Std.Tui`](README.md)
- [Layout](layout.md)
- [Application](application.md)
