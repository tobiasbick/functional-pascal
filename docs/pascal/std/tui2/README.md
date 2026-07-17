# Std.Tui2

`Std.Tui2` currently provides the value types used for terminal UI geometry.

```pascal
program Geometry;

uses Std.Tui2;

begin
  var Bounds: TuiRect := TuiRect.Create(2, 3, 8, 5);
  var Inside: boolean := Bounds.Contains(TuiPoint.Create(9, 7));
end.
```

Coordinates are zero-based: `(0, 0)` is the upper-left cell, X grows to the right, and Y grows downward.

## Quick reference

| Symbol | Description |
|--------|-------------|
| `TuiPoint` | A coordinate with `X` and `Y`. |
| `TuiSize` | A non-negative extent with `Width` and `Height`. |
| `TuiRect` | A rectangle with `X`, `Y`, `Width`, and `Height`. |
| `TuiColor` | A CRT, ANSI-256, or RGB terminal color value. |
| `TuiStyleRole` | A semantic role for a painted cell. |
| `TuiStyle` | Foreground, background, and text attributes. |
| `TuiCell` | A glyph with a semantic style role. |
| `TuiPalette` | An immutable mapping from semantic roles to styles. |
| `TuiPoint.Create(X, Y)` | Creates a point. |
| `TuiSize.Create(Width, Height)` | Creates a non-negative size. |
| `TuiRect.Create(X, Y, Width, Height)` | Creates a rectangle from its stored fields. |
| `TuiRect.FromEdges(Left, Top, Right, Bottom)` | Creates a rectangle from exclusive edges. |
| `TuiRect.FromPointSize(Position, Size)` | Creates a rectangle from a point and size. |
| `TuiRect.FromCorners(TopLeft, BottomRight)` | Creates a rectangle from exclusive corners. |

## `TuiPoint`

`TuiPoint` is a record with public `X` and `Y` integer fields. `TuiPoint.Create(X, Y)` creates a point and accepts all integer coordinates.

## `TuiSize`

`TuiSize` is a record with public `Width` and `Height` integer fields. `TuiSize.Create` rejects negative dimensions. A size is empty when either dimension is zero:

```pascal
var Empty: boolean := TuiSize.Create(0, 4).IsEmpty();
```

## `TuiRect`

`TuiRect` uses `X`, `Y`, `Width`, and `Height`. Width and height must be non-negative. Its right and bottom edges are exclusive:

```text
right  = x + width
bottom = y + height
```

`TuiRect.Create` rejects dimensions that are negative and coordinates whose right or bottom edge would overflow an integer. `FromEdges` and `FromCorners` use exclusive right and bottom values and reject reversed or unrepresentable extents.

| Method | Result |
|--------|--------|
| `Right()` | Exclusive right edge. |
| `Bottom()` | Exclusive bottom edge. |
| `IsEmpty()` | True when width or height is zero. |
| `Contains(Point)` | True for points inside the half-open rectangle. |
| `Intersects(Other)` | True when two non-empty rectangles overlap. |
| `Intersect(Other)` | The overlapping rectangle, or an empty rectangle. |

For a rectangle at `(2, 3)` with size `(8, 5)`, `(2, 3)` through `(9, 7)` are inside; `(10, 7)` and `(9, 8)` are outside.

```pascal
var Bounds: TuiRect := TuiRect.FromEdges(2, 3, 10, 8);
var SameBounds: TuiRect := TuiRect.FromPointSize(
  TuiPoint.Create(2, 3),
  TuiSize.Create(8, 5)
);
```

## Cell values

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts `0..15`, while
`FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal
var Foreground: TuiColor := TuiColor.FromCrt(14);
var Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
var Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
var Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and `Inverse` flags. A
`TuiCell` is a value only; drawing and glyph validation are not exposed by `Std.Tui2` yet.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves one style and
`WithRole` returns a copy with one replacement, leaving the original palette unchanged.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

## Implementation (contributors)

`Std.Tui2` is a source-level standard-library facade in [`lib/Std/Tui2.fpas`](../../../../lib/Std/Tui2.fpas). Its geometry records are implemented in focused private units under [`lib/Std/Tui2/Geometry/`](../../../../lib/Std/Tui2/Geometry/). It is exported by [`lib/stdlib.fpasprj`](../../../../lib/stdlib.fpasprj).

## See also

- [Standard library reference](../README.md)
- [Units](../../program-structure/units.md)
