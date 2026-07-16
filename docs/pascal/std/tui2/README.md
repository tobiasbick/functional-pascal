# Std.Tui2

`Std.Tui2` currently provides the value types used for terminal UI geometry.

```pascal
program Geometry;

uses Std.Tui2;

begin
  var Bounds: TuiRect := TuiRectOf(2, 3, 8, 5);
  var Inside: boolean := Bounds.Contains(TuiPointAt(9, 7));
end.
```

Coordinates are zero-based: `(0, 0)` is the upper-left cell, X grows to the right, and Y grows downward.

## Quick reference

| Symbol | Description |
|--------|-------------|
| `TuiPoint` | A coordinate with `X` and `Y`. |
| `TuiSize` | A non-negative extent with `Width` and `Height`. |
| `TuiRect` | A rectangle with `X`, `Y`, `Width`, and `Height`. |
| `TuiPointAt(X, Y)` | Creates a point. |
| `TuiSizeOf(Width, Height)` | Creates a non-negative size. |
| `TuiRectOf(X, Y, Width, Height)` | Creates a rectangle with checked edges. |

## `TuiPoint`

`TuiPoint` is a record with public `X` and `Y` integer fields. `TuiPointAt(X, Y)` creates a point and accepts all integer coordinates.

## `TuiSize`

`TuiSize` is a record with public `Width` and `Height` integer fields. `TuiSizeOf` rejects negative dimensions. A size is empty when either dimension is zero:

```pascal
var Empty: boolean := TuiSizeOf(0, 4).IsEmpty();
```

## `TuiRect`

`TuiRect` uses `X`, `Y`, `Width`, and `Height`. Width and height must be non-negative. Its right and bottom edges are exclusive:

```text
right  = x + width
bottom = y + height
```

`TuiRectOf` rejects dimensions that are negative and coordinates whose right or bottom edge would overflow an integer.

| Method | Result |
|--------|--------|
| `Right()` | Exclusive right edge. |
| `Bottom()` | Exclusive bottom edge. |
| `IsEmpty()` | True when width or height is zero. |
| `Contains(Point)` | True for points inside the half-open rectangle. |
| `Intersects(Other)` | True when two non-empty rectangles overlap. |
| `Intersect(Other)` | The overlapping rectangle, or an empty rectangle. |

For a rectangle at `(2, 3)` with size `(8, 5)`, `(2, 3)` through `(9, 7)` are inside; `(10, 7)` and `(9, 8)` are outside.

## Implementation (contributors)

`Std.Tui2` is a source-level standard-library unit in [`lib/Std/Tui2.fpas`](../../../../lib/Std/Tui2.fpas). It is exported by [`lib/stdlib.fpasprj`](../../../../lib/stdlib.fpasprj).

## See also

- [Standard library reference](../README.md)
- [Units](../../program-structure/units.md)
