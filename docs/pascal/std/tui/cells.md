# `Std.Tui` cells and styles

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts
`0..15`; `FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal
var Foreground: TuiColor := TuiColor.FromCrt(14);
var Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
var Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
var Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and
`Inverse` flags. `TuiCell.Create` requires exactly one non-zero-width extended
grapheme cluster and stores its terminal column width (`1` or `2`).
`Width()` returns that stored value.

The cell stores a semantic `TuiStyleRole`; palette lookup supplies concrete
colors. Continuation cells for wide glyphs remain private surface state and are
not part of the public cell value.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves
one style and `WithRole` returns a copy with one replacement, leaving the
original palette unchanged.

The general roles (`Normal`, `Focused`, `Frame`, and `Title`) style the desktop
and ordinary windows. Menus use `MenuNormal`, `MenuDisabled`, `MenuShortcut`,
`MenuSelected`, and `MenuSelectedShortcut`; status lines use `StatusNormal`,
`StatusDisabled`, `StatusShortcut`, and `StatusSelected`. The menu bar and
status line paint their complete row with their normal role, so their chrome
remains distinct from the desktop.

Dialog painting uses separate `DialogNormal`,
`DialogFrame`, `DialogTitle`, `DialogInput`, `DialogInputFocused`,
`DialogButton`, `DialogButtonFocused`, and `DialogShadow` roles. A theme can
therefore use a gray dialog surface over a blue desktop without changing
ordinary window content.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

A palette is ordinary public FPAS data. Applications may start from
`TuiPalette.Default()`, replace only the roles they need with `WithRole`, or
construct all role styles explicitly with `TuiPalette.Create`. This is the
theme extension boundary; no theme registry or fixed set of color names is
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
