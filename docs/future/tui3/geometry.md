# Std.Tui3 geometry and coordinate spaces

Geometry values follow the Tui2 rules already proven in
[`docs/pascal/std/tui2/`](../../pascal/std/tui2/README.md): zero-based coordinates,
non-negative sizes, and half-open rectangles (`right = x + width`, `bottom = y + height`).

Tui3 reuses those value semantics. It does not reuse retained parent-pointer trees.

## Coordinate spaces

| Space | Meaning |
| --- | --- |
| Local | Relative to an element's content origin after layout. |
| Parent | Child bounds relative to the parent's content rectangle. |
| Application | Resolved coordinates relative to the desktop origin. |
| Console | One-based, screen-absolute coordinates only at the terminal boundary. |

Layout produces parent-relative rectangles for each element node. Paint and hit-testing
resolve application rectangles and ancestor clips from the laid-out tree for the current
frame. Applications do not store screen-absolute rectangles in the model for ordinary
controls.

Conversion between zero-based Tui coordinates and one-based `Std.Console` coordinates
occurs exactly once at the terminal boundary. Incoming mouse coordinates use the inverse
conversion.

## Clipping and hit-testing

- Each child clip is the intersection of its resolved rectangle, parent clip, and active
  modal subtree (the foremost dialog overlay when present).
- Empty clips skip paint and hit-testing.
- Hit-testing uses the same half-open rectangles as painting.
- The topmost visible, enabled, hit-testable interactive element wins.
- Pointer capture, if introduced, may route subsequent pointer messages outside the
  captured rectangle; painting remains clipped.

No separate coordinate convention is permitted for custom paint elements.

## Model vs geometry

Resolved rectangles are **frame outputs**, not application model fields. The model stores
logical state (which dialog is open, selected index, scroll offset). Geometry is computed
from `View(Model)` and the current application size.
