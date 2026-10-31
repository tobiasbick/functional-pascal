# `Std.Ui.Terminal` layout

Public layout values are `TuiSizePolicy`, `TuiAlignment`, `TuiMargins`,
`TuiLayoutSettings`, `TuiMeasureSpec`, `TuiMeasureResult`, `TuiSpacer`, and
`TuiLayoutFit`. `TuiMeasure(Node, Spec)` is pure.

Arrangement creates an internal host-owned frame containing preorder parent
indices, bounds, clips, and subtree boundaries. Applications cannot create,
retain, or inspect this frame. Painting reads the matching frame and does not
remeasure. The host reuses geometry while the viewport and the tree's geometry
inputs match. Geometry changes arrange into a reusable staging frame. The host
publishes its tree/frame pair only after validation and rendering succeed.
Each host retains only its last tree, geometry key, and two frame buffers;
closing the host releases their contents.

`Row` and `Column` reserve spacing, assign child minimum extents, grow in stable
child order toward preferred extents, then distribute remaining cells through
`Stretch` or `Expanding`. A `TuiLayoutSettings` wrapper applies margins and
aligns its child independently per axis with `Leading`, `Center`, `Trailing`, or
`Fill`. `Expanding` raises the wrapper's maximum on the selected axes; combined
with `Fill`, the child consumes the corresponding allocated content area.
`Settings.WithFixedWidth(Width)` and `Settings.WithFixedHeight(Height)` return
copies whose total horizontal or vertical extent, including margins, is fixed
to the supplied non-negative cell count. Fixed axes do not receive remaining
space from a `Row` or `Column`; negative extents panic.

`Panel` and `Overlay` arrange their inset children as a vertical column without
spacing. A direct desktop `Overlay` is centered at its preferred size; a
`Panel` participates in ordinary layout.

## Responsive overlays

`TuiOverlaySizeRule.Create(Minimum, MarginCells, MarginPercent)` describes a
centered overlay independently of its contents. `Minimum` is its desired outer
frame size, `MarginCells` is the minimum gap on each side, and `MarginPercent`
is the desired percentage of the viewport on each side, from 0 to 50 inclusive.
For each axis the rule chooses the larger of the minimum and the viewport minus
the two percentage margins, capped to preserve the cell margins. When a screen
cannot fit the minimum and those margins, the margins shrink first; when even
the minimum does not fit, the overlay uses the available extent. Percentage
rounding is down. The rule is re-resolved when the viewport changes.

Pass the rule to `MakeResponsiveOverlay(Title, Rule, Children)` directly under
the desktop. It retains ordinary overlay stacking, focus and pointer behavior.
`Rule.Resolve(Size)` exposes the same outer dimensions to applications that
need to paginate or adapt their content. For example, a minimum of 72 × 17,
four-cell margins and 25% per side resolves to 72 × 17 at 80 × 25 and
100 × 30 at 200 × 60.

The rule sizes the frame; it does not automatically scroll or reflow content.
Applications must adapt or provide scrollable content and keep controls
reachable when the available space is smaller than their preferred size.

`TextArea` measures to a minimum of one cell, prefers its widest logical line
and logical-line count, and permits expansion in both axes. A direct `TextArea`
participates in stable remaining-space distribution, so its arranged bounds
become its editing viewport.

When a terminal is smaller than the combined minimum, children retain their
minimum geometry and the canvas clips overflow. `TuiLayoutFitFor` reports that
condition without changing arranged geometry. `TuiPaintTooSmallOverlay` can
replace a working surface with a notice.

## See also

- [`Std.Ui.Terminal`](README.md)
- [Geometry](geometry.md)
- [Elements](elements.md)
- [Text area](text-area.md)
- [Application](application.md)
