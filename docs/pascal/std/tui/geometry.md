# `Std.Ui.Terminal` geometry

Examples use local aliases such as `type TuiRect = Tui.TuiRect;` after
`uses Std.Ui.Terminal as Tui;`, supplied by their named documentation context.

`TuiPoint`, `TuiSize`, and `TuiRect` are immutable value records. Coordinates
are zero-based. Sizes and rectangle extents must be non-negative. Rectangles
are half-open: `right = x + width` and `bottom = y + height`.
An empty rectangle shares no cells with any rectangle, so `Intersects` returns
false whenever either operand is empty.

```pascal id=example-1 fragment context=tui-basic
const Bounds: TuiRect := TuiRect.FromEdges(2, 3, 10, 8);
const Inside: boolean := Bounds.Contains(TuiPoint.Create(9, 7));
const Content: TuiRect := Bounds.Inset();
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

- [`Std.Ui.Terminal`](README.md)
- [Layout](layout.md)
