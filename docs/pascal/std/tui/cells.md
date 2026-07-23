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
grapheme cluster. `Width()` is one or two terminal columns through
`Std.Console.GraphemeWidth`.

The cell stores a semantic `TuiStyleRole`; palette lookup supplies concrete
colors. Continuation cells for wide glyphs remain private surface state and are
not part of the public cell value.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves
one style and `WithRole` returns a copy with one replacement, leaving the
original palette unchanged.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

## See also

- [`Std.Tui`](README.md)
- [Layout](layout.md)
- [Application](application.md)
