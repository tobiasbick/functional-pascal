# `Std.Tui` geometry

`TuiPoint`, `TuiSize`, and `TuiRect` are immutable value records. Coordinates
are zero-based. Sizes and rectangle extents must be non-negative. Rectangles
are half-open: `right = x + width` and `bottom = y + height`.

```pascal
uses Std.Tui as Tui;
uses Std.Tui.Geometry as Geometry;

const Bounds: Tui.TuiRect := Geometry.TuiRectFromEdges(2, 3, 10, 8);
const Inside: boolean := Geometry.TuiRectContains(Bounds, Geometry.TuiPointCreate(9, 7));
const Content: Tui.TuiRect := Geometry.TuiRectInset(Bounds);
```

| Symbol | Purpose |
| --- | --- |
| `Geometry.TuiPointCreate(X, Y)` | Creates a point; all integer coordinates are accepted. |
| `Geometry.TuiSizeCreate(Width, Height)` | Creates a size; rejects negative dimensions. |
| `Geometry.TuiSizeIsEmpty(Size)` | True when width or height is zero. |
| `Geometry.TuiRectCreate(X, Y, Width, Height)` | Creates a rectangle; rejects negative or overflowing extents. |
| `Geometry.TuiRectFromEdges(Left, Top, Right, Bottom)` | Creates a rectangle from exclusive edges. |
| `Geometry.TuiRectFromPointSize(Position, Size)` | Creates a rectangle from a point and size. |
| `Geometry.TuiRectFromCorners(TopLeft, BottomRight)` | Creates a rectangle from exclusive corners. |
| `Geometry.TuiRectRight(Bounds)` / `Geometry.TuiRectBottom(Bounds)` | Returns exclusive edges. |
| `Geometry.TuiRectIsEmpty(Bounds)` | True when width or height is zero. |
| `Geometry.TuiRectContains(Bounds, Point)` | Tests half-open containment. |
| `Geometry.TuiRectIntersects(Bounds, Other)` / `Geometry.TuiRectIntersect(Bounds, Other)` | Tests overlap and returns the intersection rectangle. |
| `Geometry.TuiRectInset(Bounds)` | Shrinks by one cell on every side, clamped to empty. |

For a rectangle at `(2, 3)` with size `(8, 5)`, points `(2, 3)` through
`(9, 7)` are inside; `(10, 7)` and `(9, 8)` are outside.

## See also

- [`Std.Tui`](README.md)
- [Layout](layout.md)
