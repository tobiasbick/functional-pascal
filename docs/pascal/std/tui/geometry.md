# `Std.Tui` geometry

`TuiPoint`, `TuiSize`, and `TuiRect` are immutable value records. Coordinates
are zero-based. Sizes and rectangle extents must be non-negative. Rectangles
are half-open: `right = x + width` and `bottom = y + height`.

```pascal
var Bounds: TuiRect := TuiRect.FromEdges(2, 3, 10, 8);
var Inside: boolean := Bounds.Contains(TuiPoint.Create(9, 7));
var Content: TuiRect := Bounds.Inset();
```

| Symbol | Purpose |
| --- | --- |
| `TuiPoint.Create(X, Y)` | Creates a point; all integer coordinates are accepted. |
| `TuiSize.Create(Width, Height)` | Creates a size; rejects negative dimensions. |
| `TuiSize.IsEmpty()` | True when width or height is zero. |
| `TuiRect.Create(X, Y, Width, Height)` | Creates a rectangle; rejects negative or overflowing extents. |
| `TuiRect.FromEdges(Left, Top, Right, Bottom)` | Creates a rectangle from exclusive edges. |
| `TuiRect.FromPointSize(Position, Size)` | Creates a rectangle from a point and size. |
| `TuiRect.FromCorners(TopLeft, BottomRight)` | Creates a rectangle from exclusive corners. |
| `Right()` / `Bottom()` | Returns exclusive edges. |
| `IsEmpty()` | True when width or height is zero. |
| `Contains(Point)` | Tests half-open containment. |
| `Intersects(Other)` / `Intersect(Other)` | Tests overlap and returns the intersection rectangle. |
| `Inset()` | Shrinks by one cell on every side, clamped to empty. |

For a rectangle at `(2, 3)` with size `(8, 5)`, points `(2, 3)` through
`(9, 7)` are inside; `(10, 7)` and `(9, 8)` are outside.

## See also

- [`Std.Tui`](README.md)
- [Layout](layout.md)
