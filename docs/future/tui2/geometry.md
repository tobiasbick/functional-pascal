# Std.Tui2 geometry and coordinate spaces

## Coordinate convention

Std.Tui2 uses zero-based coordinates. `(0, 0)` is the upper-left cell, X increases to the right, and Y increases downward.

`Std.Console` cell operations remain one-based. Conversion occurs exactly once at the final console rendering boundary. Incoming one-based mouse coordinates are converted exactly once when they enter Std.Tui2.

## Value contracts

```text
TuiPoint  = x, y
TuiSize   = width, height
TuiRect   = x, y, width, height
```

- Sizes are non-negative.
- Empty sizes and rectangles are valid internal values.
- Public constructors reject negative width or height.
- Right and bottom edges are exclusive.
- `Contains(Rect, Point)` uses `x <= point.x < x + width` and the equivalent Y rule.
- Intersection may return an empty rectangle.
- Checked arithmetic reports overflow instead of wrapping coordinates.

## Construction API

Geometry values are constructed through static record functions. FPAS does not use overloads, so each alternative representation has a distinct name:

```pascal
TuiPoint.Create(X, Y)
TuiSize.Create(Width, Height)

TuiRect.Create(X, Y, Width, Height)
TuiRect.FromEdges(Left, Top, Right, Bottom)
TuiRect.FromPointSize(Position, Size)
TuiRect.FromCorners(TopLeft, BottomRight)
```

- `Create` accepts the fields stored by the record.
- `FromEdges` treats `Right` and `Bottom` as exclusive edges and rejects reversed edges.
- `FromPointSize` combines a `TuiPoint` origin with a `TuiSize` extent.
- `FromCorners` treats `BottomRight` as the exclusive corner and rejects reversed coordinates.
- Copying an existing geometry record uses ordinary value assignment.
- Free functions such as `TuiPointAt`, `TuiSizeOf`, and `TuiRectOf` are provisional implementation details to remove during the restructure.

The static-call syntax is specified by the [static record function plan](../static-record-functions.md).

## Coordinate spaces

| Space | Meaning |
| --- | --- |
| Local | Relative to a view's content origin. |
| Parent | A child's bounds relative to its parent's content rectangle. |
| Application | Resolved coordinates relative to the desktop origin. |
| Console | One-based, screen-absolute coordinates used only at the terminal boundary. |

View bounds are parent-relative. The registry caches resolved application rectangles and clips for paint and hit-testing. Applications do not store screen-absolute rectangles in child controls.

Frames own an outer rectangle and expose an inner content rectangle. Child layout always receives the content rectangle.

## Clipping and hit-testing

- Each child clip is the intersection of its resolved rectangle, parent clip, and active modal boundary.
- Empty clips skip paint and hit-testing.
- Hit-testing uses the same resolved half-open rectangles as painting.
- The topmost visible, enabled, hit-testable view wins.
- Pointer capture may route subsequent pointer events outside the captured view's rectangle, but painting remains clipped.

No separate coordinate convention is permitted for custom views.
