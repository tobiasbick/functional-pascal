# Std.Tui2 geometry and coordinate spaces

The implemented geometry values and constructor rules are documented in the current
[`Std.Tui2` reference](../../pascal/std/tui2/README.md). This plan covers their future use by the
view tree, renderer, and input router.

## Coordinate spaces

| Space | Meaning |
| --- | --- |
| Local | Relative to a view's content origin. |
| Parent | A child's bounds relative to its parent's content rectangle. |
| Application | Resolved coordinates relative to the desktop origin. |
| Console | One-based, screen-absolute coordinates used only at the terminal boundary. |

View bounds are parent-relative. The headless implementation resolves application rectangles and
ancestor clips from the retained tree for both paint and hit-testing. Applications do not store
screen-absolute rectangles in child controls. Caching those resolved values remains an interactive
runtime optimization rather than a separate coordinate model.

Conversion between zero-based Tui2 coordinates and one-based `Std.Console` coordinates occurs
exactly once at the terminal boundary. Incoming mouse coordinates follow the inverse conversion.

Frames own an outer rectangle and expose an inner content rectangle. Child layout always receives the content rectangle.

## Clipping and hit-testing

- Each child clip is the intersection of its resolved rectangle, parent clip, and active modal boundary.
- Empty clips skip paint and hit-testing.
- Hit-testing uses the same resolved half-open rectangles as painting.
- The topmost visible, enabled, hit-testable view wins.
- Pointer capture may route subsequent pointer events outside the captured view's rectangle, but painting remains clipped.

No separate coordinate convention is permitted for custom views.

Implemented headless behavior uses stable retained subtree order for back-to-front painting and the
reverse order for topmost hit-testing. Only registered custom views are targets at this stage;
containers contribute bounds and clipping but are not returned as hits.
