# Std.Tui2 layout model

Std.Tui2 uses layouts to position and resize child views when their container or terminal changes size. Applications describe size preferences and relationships instead of recalculating every child rectangle manually.

Absolute bounds remain available for specialized views and compatibility with direct terminal drawing. They are not the default composition mechanism for dialogs and application chrome.

## Size contract

Every view and nested layout reports three sizes:

| Value | Meaning |
| --- | --- |
| Minimum size | Smallest useful or supported extent. |
| Preferred size | Natural extent for current content. |
| Maximum size | Largest extent the item should receive. |

The horizontal and vertical dimensions have independent policies:

| Policy | Meaning |
| --- | --- |
| `Fixed` | Use the preferred extent; do not grow or shrink. |
| `Minimum` | Preferred extent is the minimum; growth is allowed but not requested. |
| `Maximum` | Preferred extent is the maximum; shrinking is allowed. |
| `Preferred` | Preferred extent is ideal; growing and shrinking are allowed. |
| `Expanding` | Preferred extent is useful, and additional space is actively requested. |

The initial public value types are:

```text
TuiSizePolicyKind
TuiSizePolicy
TuiAlignment
TuiMargins
TuiMeasureSpec
TuiMeasureResult
TuiSpacer
TuiLayoutItem
```

`TuiSizePolicy` contains separate horizontal and vertical policies. A view starts with a preferred
policy on both axes and a fixed zero size hint; both values are retained registry properties that an
application may replace.

Implemented: `TuiSizePolicy.Create(Horizontal, Vertical)` represents independent axes, while
`Fixed`, `Minimum`, `Maximum`, `Preferred`, and `Expanding` construct uniform policies. Recursive
measurement and horizontal or vertical allocation evaluate these policies.

## Layout items

A layout manages an ordered list of `TuiLayoutItem` values. An item contains exactly one of:

- a child `TuiView`;
- a nested `TuiLayout`;
- a fixed or expanding `TuiSpacer`.

An item also carries its alignment and stretch factor. Stretch factors are relative weights used to distribute additional space along the layout's main direction.

The view tree owns live views. Layouts arrange views but do not create a second ownership hierarchy.

Implemented foundation: `TuiSpacer.Fixed` and `Expanding` validate their main-axis extents.
`TuiLayoutItem.ForView`, `ForLayout`, and `ForSpacer` validate the selected live handle, alignment,
and non-negative stretch. `TuiLayoutItems` retains the ordered lists. A view can occur in one list;
removal does not destroy it because the view tree owns it. Nested layouts have exclusive ownership,
cannot also be a container root, reject cycles, and are destroyed with their parent item.

## Initial layouts

| Layout | Purpose |
| --- | --- |
| `TuiHorizontalLayout` | Arrange items in one horizontal row. |
| `TuiVerticalLayout` | Arrange items in one vertical column. |
| `TuiGridLayout` | Arrange items in rows and columns with optional spans. |
| `TuiFormLayout` | Arrange label and field pairs consistently. |
| `TuiStackedLayout` | Give multiple items the same area while selecting which one is visible. |

Layouts may contain other layouts. This is the primary mechanism for composing complex dialogs and application chrome.

Each layout supports outer margins and spacing between neighboring items. Alignment controls whether an item fills its allocated area or uses a smaller aligned rectangle.

Implemented: `TuiMargins` validates outer cell counts and provides uniform and symmetric
constructors. `TuiAlignment` combines independent `Leading`, `Center`, `Trailing`, or `Fill` choices
for both axes. A common layout has an immutable horizontal, vertical, grid, form, or stacked kind;
`TuiLayoutSettings` retains margins and spacing for each live layout. Typed handles provide
controlled `AsLayout()` conversion.

Implemented grid foundation: `TuiGridLayout` uses the same common handle identity.
`TuiGridPlacement` validates zero-based cells and positive row or column spans. `TuiGridItems`
rejects overlap and one-dimensional spacers while preserving common view and nested-layout
ownership. Rows and columns are inferred from visible placements.

Implemented form and stacked layouts: `TuiFormLayout` owns paired label and field views in inferred
two-column rows. `TuiStackedLayout` owns view or nested-layout pages, measures the maximum visible
page geometry, and arranges only its selected `CurrentIndex` page.

## Measurement and allocation

Layout runs in two directions:

1. From leaves to root, controls and nested layouts calculate minimum, preferred, and maximum sizes for a `TuiMeasureSpec`.
2. From root to leaves, each layout receives a final `TuiRect` and allocates rectangles to its items.

Allocation follows these rules:

1. Reserve margins and fixed spacing.
2. Give every item at least its applicable minimum extent when space permits.
3. Move items toward their preferred extents.
4. Distribute remaining space using stretch factors and expanding policies.
5. Never exceed an item's maximum extent.
6. Apply alignment inside the allocated rectangle.
7. Distribute indivisible cell remainders deterministically in item order.

Hidden views are excluded from measurement and allocation by default. Showing or hiding a view invalidates its parent layout.

Implemented: `TuiLayoutMeasure.Measure` combines visible view hints, policies, nested layouts,
spacers, spacing, margins, grid spans, form rows, and stacked pages.
`TuiLayoutArrange.Arrange` performs a recursive headless pass and assigns view bounds. Box slots and
grid tracks distribute stretch and expanding space in stable order while aligned items retain finite
maximum sizes. Form rows reuse the grid allocator; stacks allocate only their selected visible page.
Layouts track dirty state and propagate nested invalidation to their root. Containers expose an
explicit coalesced layout pass that also detects size changes. Scheduling that pass automatically in
the application loop remains part of the runtime phase.

## Invalidation

A layout is invalidated when an input to measurement changes, including:

- child insertion or removal;
- visibility changes;
- text or other content changes;
- minimum, preferred, or maximum size changes;
- size policy or stretch changes;
- margins or spacing changes;
- container or terminal resize.

Repeated invalidations before the next container pass are coalesced. `NeedsLayout()` reports the
pending work and `PerformLayout()` resolves the complete tree before returning. The application loop
will run this pass before repaint so drawing and hit-testing observe the same resolved rectangles.

## Layout API

The implemented layout surface is:

```pascal
TuiHorizontalLayout.Create(App: TuiApplication): TuiHorizontalLayout
TuiVerticalLayout.Create(App: TuiApplication): TuiVerticalLayout
Horizontal.AsLayout(): TuiLayout
Vertical.AsLayout(): TuiLayout
TuiLayoutSettings.SetMargins(Layout: TuiLayout; Margins: TuiMargins): boolean
TuiLayoutSettings.SetSpacing(Layout: TuiLayout; Spacing: integer): boolean
TuiLayoutMeasure.Measure(Layout: TuiLayout; Spec: TuiMeasureSpec): TuiMeasureResult
TuiLayoutArrange.Arrange(Layout: TuiLayout; Bounds: TuiRect): boolean
TuiGridLayout.Create(App: TuiApplication): TuiGridLayout
TuiGridPlacement.Cell(Row: integer; Column: integer): TuiGridPlacement
TuiGridPlacement.Create(Row: integer; Column: integer; RowSpan: integer; ColumnSpan: integer): TuiGridPlacement
TuiGridItems.Add(Grid: TuiGridLayout; Item: TuiLayoutItem; Placement: TuiGridPlacement): boolean
TuiFormLayout.Create(App: TuiApplication): TuiFormLayout
TuiFormItems.AddRow(Form: TuiFormLayout; LabelView: TuiView; FieldView: TuiView): boolean
TuiFormItems.AddRowItems(Form: TuiFormLayout; LabelItem: TuiLayoutItem; FieldItem: TuiLayoutItem): boolean
TuiStackedLayout.Create(App: TuiApplication): TuiStackedLayout
TuiStackedItems.Add(Stacked: TuiStackedLayout; Item: TuiLayoutItem): boolean
Stacked.CurrentIndex: integer

TuiLayoutItems.Add(Layout: TuiLayout; Item: TuiLayoutItem)
TuiLayoutItems.Count(Layout: TuiLayout)
TuiLayoutItems.Get(Layout: TuiLayout; Index: integer)
TuiLayoutItems.RemoveAt(Layout: TuiLayout; Index: integer)
TuiLayoutItems.Clear(Layout: TuiLayout)
TuiLayout.SetStretch(Layout: TuiLayout; Index: integer; Stretch: integer)

TuiView.SetSizePolicy(View: TuiView; Policy: TuiSizePolicy)
TuiView.Measure(View: TuiView; Spec: TuiMeasureSpec): TuiMeasureResult
```

The item-list and specialized layout functions above are implemented. Per-item stretch replacement
and control-specific measurement remain planned.

## Terminal overflow

Minimum sizes are never silently violated. When a container receives less space than its measured
minimum, its content keeps minimum geometry. The read-only `LayoutFit` property exposes `Minimum`,
`Available`, and the positive `Overflow` on each axis; `Fits()` reports whether both axes fit. A
later interactive renderer clips overflow to the container.

The implemented headless `TuiScrollView` owns a container layout and measures its unbounded preferred
size. Content is never smaller than the viewport. Its non-negative offset is clamped to the content
excess on each axis, and its layout pass positions content at the corresponding negative local
origin. Content and viewport changes invalidate the pass and clamp an offset that is no longer
reachable. Interactive clipping and input-driven scrolling remain part of the rendering and event
routing phases.

When the terminal is smaller than the desktop minimum, Std.Tui2 replaces normal desktop paint and input with a built-in too-small overlay showing current and required sizes. Only resize and quit actions remain active. Normal layout resumes automatically when the terminal is large enough.

## Dependent measurement

`TuiMeasureSpec` describes each axis as unbounded or at-most a cell count. Wrapped labels and text views calculate height from the supplied width constraint. Measurement caches include the full specification, so the same view may have different preferred heights for different widths.

Implemented foundation: `TuiMeasureConstraint.Unbounded()` and `AtMost(Limit)` represent one axis;
`TuiMeasureSpec` combines width and height constraints. `TuiMeasureResult` stores validated minimum,
preferred, and maximum sizes. Current views contribute their retained `SizeHint`; control-owned and
width-dependent measurement callbacks remain.

Dependent measurement is part of the initial layout engine rather than a later API extension.

## Deferred custom layouts

The initial implementation provides the built-in layouts above. A public callback protocol for application-defined layouts is deferred until the built-in measurement and allocation contracts are stable.
