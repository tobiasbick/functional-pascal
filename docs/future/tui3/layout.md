# Std.Tui3 layout model

Std.Tui3 lays out **element trees**, not live view registries. Measurement and allocation
are pure functions:

```text
Measure(Element, Spec) → TuiMeasureResult
Arrange(Element, Bounds) → laid-out tree with resolved rectangles
```

Applications describe size preferences on elements instead of assigning child rectangles
manually. Absolute bounds remain available for specialized custom paint elements only.

Size-policy and alignment ideas are salvaged from
[`docs/future/tui2/layout.md`](../tui2/layout.md) and the implemented Tui2 measure/arrange
algorithms. The live `TuiLayout` handle API is not.

## Size contract

Every element reports three sizes:

| Value | Meaning |
| --- | --- |
| Minimum | Smallest useful extent. |
| Preferred | Natural extent for current content. |
| Maximum | Largest extent the item should receive. |

Independent per-axis policies:

| Policy | Meaning |
| --- | --- |
| `Fixed` | Use preferred; do not grow or shrink. |
| `Minimum` | Preferred is the minimum; growth allowed but not requested. |
| `Maximum` | Preferred is the maximum; shrinking allowed. |
| `Preferred` | Preferred is ideal; growing and shrinking allowed. |
| `Expanding` | Preferred is useful; additional space is requested. |

Public value types:

```text
TuiSizePolicyKind
TuiSizePolicy
TuiAlignment
TuiMargins
TuiMeasureSpec
TuiMeasureResult
TuiSpacer
TuiLayoutFit
```

Controls advertise intrinsic sizes from content (for example button text width). Layout
elements combine children with margins, spacing, stretch, and alignment.

## Layout elements

| Element | Purpose |
| --- | --- |
| `Row` | Horizontal main axis. |
| `Column` | Vertical main axis. |
| `Grid` | Rows and columns with optional spans. |
| `Form` | Label and field pairs. |
| `Stack` | Shared area; one visible child by index. |
| `Spacer` | Fixed or expanding empty extent. |

Layout elements nest. That is the primary composition mechanism for dialogs and chrome.

## Allocation rules

Adapted from Tui2:

1. Reserve margins and inter-item spacing.
2. Assign minimum sizes.
3. Grow items toward preferred sizes.
4. Distribute remaining main-axis cells by stretch weight or `Expanding` policy.
5. Apply alignment inside each allocated slot (`Leading`, `Center`, `Trailing`, `Fill`).

Indivisible leftover cells go to earlier items or tracks in stable order. Hidden or
`None` children do not participate. When bounds fall below the combined minimum, children
keep minimum geometry and may extend beyond the container; paint clips overflow.
`TuiLayoutFit` reports shortage without mutating minimum geometry (terminal-too-small).

## Scroll

A `Scroll` element takes an offset from the model and a child. Content size is the child's
unbounded preferred size. The viewport is the allocated rectangle. Paint and hit-testing
clip to the viewport; child local origins account for the offset.

## Frame insets

`Window` and `Dialog` own an outer rectangle and expose an inner content rectangle inset
by one cell. Child layout uses the content extent. Paint draws the border and title in the
outer frame.

## Invalidation

There is no dirty-layout flag on live objects. Each MVU iteration that paints runs layout
on the current element tree for the current application size. Optimizations (skipping
unchanged subtrees) are internal and must not change observable rectangles for a given
`(Model, Size)` pair.
