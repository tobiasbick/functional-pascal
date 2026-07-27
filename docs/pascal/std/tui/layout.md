# `Std.Tui` layout

Public layout values are `TuiSizePolicy`, `TuiAlignment`, `TuiMargins`,
`TuiLayoutSettings`, `TuiMeasureSpec`, `TuiMeasureResult`, `TuiSpacer`, and
`TuiLayoutFit`. `TuiMeasure(Node, Spec)` is pure.

Arrangement creates an internal host-owned frame containing preorder parent
indices, bounds, and clips. Applications cannot create, retain, or inspect this
frame. Painting reads the matching frame and does not remeasure. The application
host replaces its previous tree/frame pair only after validation, arrangement,
and painting have succeeded.

`Row` and `Column` reserve spacing, assign child minimum extents, grow in stable
child order toward preferred extents, then distribute remaining cells through
`Stretch` or `Expanding`. A `TuiLayoutSettings` wrapper applies margins and
aligns its child independently per axis with `Leading`, `Center`, `Trailing`, or
`Fill`. `Settings.WithFixedHeight(Height)` returns a copy whose total vertical
extent is fixed to the supplied non-negative cell count.

`Panel` and `Overlay` arrange their inset children as a vertical column without
spacing. A direct desktop `Overlay` is centered at its preferred size; a
`Panel` participates in ordinary layout.

`TextArea` measures to a minimum of one cell, prefers its widest logical line
and logical-line count, and permits expansion in both axes. A direct `TextArea`
participates in stable remaining-space distribution, so its arranged bounds
become its editing viewport.

When a terminal is smaller than the combined minimum, children retain their
minimum geometry and the canvas clips overflow. `TuiLayoutFitFor` reports that
condition without changing arranged geometry. `TuiPaintTooSmallOverlay` can
replace a working surface with a notice.

## See also

- [`Std.Tui`](README.md)
- [Geometry](geometry.md)
- [Elements](elements.md)
- [Text area](text-area.md)
- [Application](application.md)
