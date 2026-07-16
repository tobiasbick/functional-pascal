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
```

`TuiSizePolicy` contains separate horizontal and vertical policies. A view supplies sensible defaults, and an application may override them for an individual view.

## Layout items

A layout manages an ordered list of `TuiLayoutItem` values. An item contains exactly one of:

- a child `TuiView`;
- a nested `TuiLayout`;
- a fixed or expanding `TuiSpacer`.

An item also carries its alignment and stretch factor. Stretch factors are relative weights used to distribute additional space along the layout's main direction.

The view tree owns live views. Layouts arrange views but do not create a second ownership hierarchy.

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

## Invalidation

A layout is invalidated when an input to measurement changes, including:

- child insertion or removal;
- visibility changes;
- text or other content changes;
- minimum, preferred, or maximum size changes;
- size policy or stretch changes;
- margins or spacing changes;
- container or terminal resize.

Repeated invalidations before the next application iteration are coalesced. Layout completes before repaint so drawing and hit-testing observe the same resolved rectangles.

## Rough API

The names below show the intended surface, not final signatures:

```pascal
TuiHorizontalLayout.New(App: TuiApplication): TuiHorizontalLayout
TuiVerticalLayout.New(App: TuiApplication): TuiVerticalLayout
TuiGridLayout.New(App: TuiApplication): TuiGridLayout
TuiFormLayout.New(App: TuiApplication): TuiFormLayout
TuiStackedLayout.New(App: TuiApplication): TuiStackedLayout

TuiLayout.AddView(Layout: TuiLayout; View: TuiView)
TuiLayout.AddLayout(Layout: TuiLayout; Child: TuiLayout)
TuiLayout.AddSpacer(Layout: TuiLayout; Spacer: TuiSpacer)
TuiLayout.SetMargins(Layout: TuiLayout; Margins: TuiMargins)
TuiLayout.SetSpacing(Layout: TuiLayout; Spacing: integer)
TuiLayout.SetStretch(Layout: TuiLayout; Index: integer; Stretch: integer)

TuiView.SetSizePolicy(View: TuiView; Policy: TuiSizePolicy)
TuiView.Measure(View: TuiView; Spec: TuiMeasureSpec): TuiMeasureResult
```

Typed layout handles require the same controlled conversion to `TuiLayout` that typed view handles use for `TuiView`.

## Terminal overflow

Minimum sizes are never silently violated. When a nested container receives less space than its measured minimum, its content keeps minimum geometry and is clipped to the container. Applications use `TuiScrollView` when clipped content must remain reachable.

When the terminal is smaller than the desktop minimum, Std.Tui2 replaces normal desktop paint and input with a built-in too-small overlay showing current and required sizes. Only resize and quit actions remain active. Normal layout resumes automatically when the terminal is large enough.

## Dependent measurement

`TuiMeasureSpec` describes each axis as unbounded or at-most a cell count. Wrapped labels and text views calculate height from the supplied width constraint. Measurement caches include the full specification, so the same view may have different preferred heights for different widths.

Dependent measurement is part of the initial layout engine rather than a later API extension.

## Deferred custom layouts

The initial implementation provides the built-in layouts above. A public callback protocol for application-defined layouts is deferred until the built-in measurement and allocation contracts are stable.
