# `Std.Ui.Terminal` cells and styles

Examples use explicit local aliases for `Tui.Name` after importing
`uses Std.Ui.Terminal as Tui;`; their named documentation contexts supply these aliases.

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts
`0..15`; `FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal id=example-1 fragment context=tui-basic
const Foreground: TuiColor := TuiColor.FromCrt(14);
const Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
const Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
const Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
const TruecolorCell: TuiCell := TuiCell.Styled('▓', Style);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and
`Inverse` flags. These flags are retained in style values, but the current
interactive terminal renderer transfers only glyphs and foreground/background
colors to `Std.Console`. The flags have no visible terminal effect.

`TuiCell.Create` requires exactly one non-zero-width extended
grapheme cluster and stores its terminal column width (`1` or `2`).
`Width()` returns that stored value.

`TuiCell.Create` stores a semantic `TuiStyleRole`; palette lookup supplies
concrete colors. `TuiCell.Styled` stores a concrete style that bypasses palette
lookup. This is useful inside `TuiCellGrid` for plots and images whose colors
are data rather than theme roles. Continuation cells for wide glyphs remain
private surface state and are not part of the public cell value.
Button cells additionally carry their semantic appearance for theme lookup and
terminal damage tracking.

`TuiWorkingSurface.Resize(Size)` replaces the mutable grid with a blank grid of
the requested size while preserving the surface handle. Existing snapshots stay
immutable, and resizing one surface does not affect other surfaces.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves
one style and `WithRole` returns a copy with one replacement, leaving the
original palette unchanged.

The RGB default palette uses a dark terminal background, restrained borders,
and blue selection accents. `Normal` styles the desktop. Panels use
`PanelNormal`, `Frame`, `PanelFocusedFrame`, and `PanelTitle`; overlays use
`OverlayNormal`, `OverlayFrame`, `OverlayFocusedFrame`, and `OverlayTitle`.
Ordinary text within each container inherits its surface role. A panel is
focused when it contains the focused control; the last desktop overlay is active
even without a focused control. Concrete cells retain their own styles.
Container roles are optional palette overrides: unset roles fall back to
`Normal`, `Frame`, or `Title` respectively, including after those styles change.
Tab bars provide optional `TabNormal`, `TabSelected`, `TabSelectedFocused`, and
`TabDisabled` styles; see [Tabs](tabs.md).
Menus use `MenuNormal`,
`MenuDisabled`, `MenuShortcut`, `MenuSelected`, and
`MenuSelectedShortcut`; status lines use
`StatusNormal`, `StatusDisabled`, `StatusShortcut`, and `StatusSelected`. The
menu bar and status line paint their complete row with their normal role, so
their chrome remains distinct from the desktop.

Buttons use `ButtonNormal`, `ButtonDefault`, `ButtonSelected`,
`ButtonDisabled`, `ButtonShortcut`, `ButtonDefaultShortcut`,
and `ButtonSelectedShortcut` as fallback roles. `TuiPalette.Buttons` also accepts
`TuiButtonColors` per semantic button kind, with normal, active, focused,
pressed, disabled, and idle-default styles. `Resolve` combines a button's kind,
state, default flag, and mnemonic. For enabled buttons, an optional
`TuiButtonColors.ShortcutForeground` overrides the mnemonic foreground. When
unset, the matching fallback shortcut role supplies the foreground. The state
style supplies the background and attributes. Disabled buttons do not highlight
their mnemonic. `WithRole` and `ForRole` operate on the fallback roles.

`Rule`, `GaugeTrack`, and `GaugeFill` style the dashboard primitives.

One-line inputs use `InputNormal`, `InputFocused`, `InputHint`, `InputCursor`,
and `InputScroll`, grouped in `TuiPalette.Inputs` as `TuiInputPalette`. Themes
should keep these roles on a common field background while changing foreground
or inverse cursor colors. `TuiInputPalette.WithRole` returns a copy with one
input role replaced, while `ForRole` resolves one input role.

```pascal id=example-2 fragment context=tui-basic
const Palette: TuiPalette := TuiPalette.Default();
const Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
const Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
const Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

A palette is ordinary public FPAS data. Applications start from
`TuiPalette.Default()` and replace the roles they need with `WithRole`. This is
the theme extension boundary; no theme registry or fixed set of color names is
required.

Use `OpenForTestWithPalette` or `RunWithPalette` to select the initial palette.
An Update function can switch it immediately:

```pascal id=example-3 fragment context=tui-palette
Cmd.SetPalette(MyPalette);
```

The next interactive frame is fully recolored even when its glyphs and
semantic roles did not change. `App.Palette()` exposes the active palette for
headless assertions.

## See also

- [`Std.Ui.Terminal`](README.md)
- [Layout](layout.md)
- [Application](application.md)
