# Std.Tui2

`Std.Tui2` provides terminal UI value types plus a headless live-object surface for applications,
actions, buttons, views, input routing, modal roots, and layouts. Horizontal, vertical, grid, form,
and stacked layouts can be measured, arranged, painted, and driven with injected input without a
terminal. Interactive terminal acquisition is not exposed yet.

```pascal
program Geometry;

uses Std.Tui2;

begin
  var Bounds: TuiRect := TuiRect.Create(2, 3, 8, 5);
  var Inside: boolean := Bounds.Contains(TuiPoint.Create(9, 7));
end.
```

Coordinates are zero-based: `(0, 0)` is the upper-left cell, X grows to the right, and Y grows downward.

## Quick reference

| Symbol | Description |
|--------|-------------|
| `TuiPoint` | A coordinate with `X` and `Y`. |
| `TuiSize` | A non-negative extent with `Width` and `Height`. |
| `TuiRect` | A rectangle with `X`, `Y`, `Width`, and `Height`. |
| `TuiColor` | A CRT, ANSI-256, or RGB terminal color value. |
| `TuiStyleRole` | A semantic role for a painted cell. |
| `TuiStyle` | Foreground, background, and text attributes. |
| `TuiCell` | A glyph with a semantic style role. |
| `TuiPalette` | An immutable mapping from semantic roles to styles. |
| `TuiSurface` | A headless retained cell surface. |
| `TuiCanvas` | A transient, clipped drawing capability for a surface. |
| `TuiApplication` | An application-scoped headless registry and lifecycle boundary. |
| `TuiApplicationInput` | Headless input injection and unhandled application input events. |
| `TuiKeyEvent` | Alias of the canonical `Std.Console.KeyEvent`. |
| `TuiPointerEvent` | A normalized pointer action, button, position, and modifiers. |
| `TuiPointerButton` | `NoButton`, `Left`, `Right`, or `Middle`. |
| `TuiCommand` | A validated positive command identity. |
| `TuiView` | A headless application-scoped view handle and action source identity. |
| `TuiCustomView` | A typed headless view with measure, paint, focus, and close lifecycle events. |
| `TuiViewInput` | Raw input events and pointer capture for one custom view. |
| `TuiAttachHandler` | Handler invoked after a custom view is attached. |
| `TuiDetachHandler` | Handler invoked before an attached custom view is released. |
| `TuiResizeHandler` | Handler receiving the previous and resolved custom-view bounds. |
| `TuiFocusHandler` | Handler invoked after a custom view gains focus. |
| `TuiBlurHandler` | Handler invoked after a custom view loses focus. |
| `TuiCloseRequestHandler` | Handler that accepts or rejects an explicit custom-view close. |
| `TuiClosedHandler` | Handler invoked before an explicitly closed custom-view handle becomes stale. |
| `TuiContainer` | A headless owner with ordered children and explicit Z-order changes. |
| `TuiDesktop` | The root container with hit-testing and focus traversal. |
| `TuiModal` | A scoped modal-root handle with explicit LIFO closing. |
| `TuiLayout` | A headless application-scoped layout identity. |
| `TuiMeasureConstraint` | An unbounded or at-most constraint for one measurement axis. |
| `TuiMeasureSpec` | Width and height constraints passed into measurement. |
| `TuiMeasureResult` | Minimum, preferred, and maximum measured sizes. |
| `TuiSizePolicy` | Independent horizontal and vertical layout policy. |
| `TuiMargins` | Non-negative outer layout margins. |
| `TuiAlignment` | Independent horizontal and vertical item alignment. |
| `TuiSpacer` | A fixed or expanding empty layout extent. |
| `TuiLayoutItem` | A validated view, nested-layout, or spacer description. |
| `TuiLayoutItems` | Ordered live item-list operations for a layout. |
| `TuiLayoutFit` | Minimum, available, and overflowing container extents. |
| `TuiLayoutDirection` | Horizontal or vertical main-axis direction. |
| `TuiLayoutKind` | Horizontal, vertical, grid, form, or stacked layout dispatch kind. |
| `TuiLayoutSettings` | Live margins and spacing operations. |
| `TuiHorizontalLayout` | Typed horizontal layout handle. |
| `TuiVerticalLayout` | Typed vertical layout handle. |
| `TuiGridPlacement` | A zero-based grid cell with row and column spans. |
| `TuiGridLayout` | Typed row-and-column layout handle. |
| `TuiGridItems` | Grid item placement and removal operations. |
| `TuiFormLayout` | Typed two-column label-and-field layout handle. |
| `TuiFormItems` | Form row insertion, lookup, and removal operations. |
| `TuiStackedLayout` | Typed shared-area page layout with a current-index property. |
| `TuiStackedItems` | Stacked page insertion, lookup, and removal operations. |
| `TuiLayoutMeasure` | Recursive headless layout measurement. |
| `TuiLayoutArrange` | Recursive headless rectangle allocation. |
| `TuiAction` | A reusable operation with live properties and one `OnExecute` event. |
| `TuiButton` | A headless semantic button with an optional action and one `OnClick` event. |
| `TuiScrollView` | A headless viewport that keeps oversized layout content reachable by offset. |
| `TuiPoint.Create(X, Y)` | Creates a point. |
| `TuiSize.Create(Width, Height)` | Creates a non-negative size. |
| `TuiRect.Create(X, Y, Width, Height)` | Creates a rectangle from its stored fields. |
| `TuiRect.FromEdges(Left, Top, Right, Bottom)` | Creates a rectangle from exclusive edges. |
| `TuiRect.FromPointSize(Position, Size)` | Creates a rectangle from a point and size. |
| `TuiRect.FromCorners(TopLeft, BottomRight)` | Creates a rectangle from exclusive corners. |

## `TuiPoint`

`TuiPoint` is a record with public `X` and `Y` integer fields. `TuiPoint.Create(X, Y)` creates a point and accepts all integer coordinates.

## `TuiSize`

`TuiSize` is a record with public `Width` and `Height` integer fields. `TuiSize.Create` rejects negative dimensions. A size is empty when either dimension is zero:

```pascal
var Empty: boolean := TuiSize.Create(0, 4).IsEmpty();
```

## `TuiRect`

`TuiRect` uses `X`, `Y`, `Width`, and `Height`. Width and height must be non-negative. Its right and bottom edges are exclusive:

```text
right  = x + width
bottom = y + height
```

`TuiRect.Create` rejects dimensions that are negative and coordinates whose right or bottom edge would overflow an integer. `FromEdges` and `FromCorners` use exclusive right and bottom values and reject reversed or unrepresentable extents.

| Method | Result |
|--------|--------|
| `Right()` | Exclusive right edge. |
| `Bottom()` | Exclusive bottom edge. |
| `IsEmpty()` | True when width or height is zero. |
| `Contains(Point)` | True for points inside the half-open rectangle. |
| `Intersects(Other)` | True when two non-empty rectangles overlap. |
| `Intersect(Other)` | The overlapping rectangle, or an empty rectangle. |

For a rectangle at `(2, 3)` with size `(8, 5)`, `(2, 3)` through `(9, 7)` are inside; `(10, 7)` and `(9, 8)` are outside.

```pascal
var Bounds: TuiRect := TuiRect.FromEdges(2, 3, 10, 8);
var SameBounds: TuiRect := TuiRect.FromPointSize(
  TuiPoint.Create(2, 3),
  TuiSize.Create(8, 5)
);
```

## Cell values

`TuiColor` has distinct constructors for each representation. `FromCrt` accepts `0..15`, while
`FromAnsi256` and every `FromRgb` channel accept `0..255`.

```pascal
var Foreground: TuiColor := TuiColor.FromCrt(14);
var Background: TuiColor := TuiColor.FromRgb(10, 20, 30);
var Style: TuiStyle := TuiStyle.FromColors(Foreground, Background);
var Cell: TuiCell := TuiCell.Create('X', TuiStyleRole.Focused);
```

`TuiStyle.Create` additionally accepts `Bold`, `Dim`, `Underline`, and `Inverse` flags. A
`TuiCell.Create` requires exactly one non-zero-width extended grapheme cluster and its `Width()` is
one or two terminal columns. `TuiSurface.Create` owns a zero-based retained cell grid;
`TryCellAt` returns `None` outside the grid and on a wide-glyph continuation. Overwriting either
column clears the complete previous wide glyph. `TuiCanvas.Create(Surface, Bounds)` clips drawing
to `Bounds`; `PutCell`, `FillRect`, and `WriteText` use zero-based surface coordinates.

`WriteText` segments extended grapheme clusters before drawing, so combined and joined glyphs use
the same width and continuation rules as direct `PutCell` calls.

## `TuiPalette`

`TuiPalette.Default()` provides the standard semantic colors. `ForRole` resolves one style and
`WithRole` returns a copy with one replacement, leaving the original palette unchanged.

```pascal
var Palette: TuiPalette := TuiPalette.Default();
var Warning: TuiStyle := Palette.ForRole(TuiStyleRole.Warning);
var Custom: TuiStyle := TuiStyle.FromColors(TuiColor.FromRgb(255, 128, 0), TuiColor.FromCrt(0));
var Updated: TuiPalette := Palette.WithRole(TuiStyleRole.Accent, Custom);
```

## Headless applications and lifecycle events

`TuiApplication.OpenForTest(Size)` creates an application-scoped registry without changing terminal
modes. Its read-only `Size` property retains that extent. `ResizeForTest(Size)` changes it and
immediately resizes an existing desktop root. `Start`, `Tick`, and `Close` expose deterministic
lifecycle boundaries for headless code. `Close` is idempotent and invalidates every live handle
owned by the application.

```pascal
var App: TuiApplication := TuiApplication.OpenForTest(TuiSize.Create(80, 25));
App.OnStart := procedure(Sender: TuiApplication) begin ... end;
App.OnTick := procedure(Sender: TuiApplication; DeltaMilliseconds: integer) begin ... end;
App.OnStop := procedure(Sender: TuiApplication) begin ... end;
App.Start();
App.Tick(16);
App.Close()
```

`RunIterations(IterationCount, DeltaMilliseconds)` is the bounded headless loop. It starts the
application once when needed, runs at most the requested non-negative number of iterations, and
leaves the application open when that budget is exhausted. Each iteration drains posted callbacks,
performs invalid desktop layout, delivers pending view lifecycle notifications, repaints an invalid
headless surface, invokes `OnTick`, and drains callbacks again. A negative iteration count or tick
delta is rejected.

`Quit()` requests orderly shutdown while `RunIterations` is active. The current callback returns,
the remaining iteration phases are skipped, `OnStop` runs once, and the application closes. Calling
`Quit` while no loop is active closes immediately. A posted quit request is observed before layout
and `OnTick`.

`Post(Handler)` appends a parameterless callback and returns `true` while the application is open.
`Tick` drains pending callbacks in FIFO order before and after `OnTick`; callbacks posted while a drain
is in progress run in the next drain. Closing an application discards callbacks that have not started,
and `Post` returns `false` once shutdown has begun. Access through the stale handle after close is
rejected.

The lifecycle members are single-handler [record events](../../language/types/record-events.md).
Assigning another compatible handler replaces the previous handler; assigning `nil` clears it.
Copied application handles resolve the same registry state. `Tag` is a read-write live property.

## Headless input routing

`App.Input` returns the application's input capability. `InjectKeyForTest` queues the canonical
`Std.Console.KeyEvent`; `InjectPointerForTest` queues a `TuiPointerEvent`. Injected input is FIFO,
and each `Tick` or bounded loop iteration routes at most one queued value after paint and before
`OnTick`.

```pascal
var AppInput: TuiApplicationInput := App.Input;
var Key: TuiKeyEvent := record
  kind := TuiKeyKind.Enter;
  ch := '';
  shift := false;
  ctrl := false;
  alt := false;
  meta := false;
end;
AppInput.InjectKeyForTest(Key)
```

Keys first reach the focused eligible custom view. Pointer input first reaches a captured eligible
view, otherwise it hit-tests the topmost eligible custom view. A pointer-down focuses its hit target.
An unconsumed value then reaches `App.Input.OnKey` or `App.Input.OnPointer`. Raw handlers return
`true` to consume the value and `false` to continue routing.

Application fallback handlers omit a sender because application state can be captured directly:

```pascal
AppInput.OnKey :=
  function(Key: TuiKeyEvent): boolean
  begin
    return false
  end;
```

Custom-view handlers and capture are exposed through `Custom.Input`:

```pascal
var Input: TuiViewInput := Custom.Input;
Input.OnPointer :=
  function(Sender: TuiView; Pointer: TuiPointerEvent): boolean
  begin
    return true
  end;
Input.CapturePointer()
```

`CapturePointer` accepts only an attached, visible, enabled custom view inside the active modal
root. `ReleasePointer` returns whether that view owned capture. Destruction and a modal transition
that excludes the captured view release capture automatically. Copied input handles address the
same registry-backed events and capture state.

## Headless views

`TuiView.Create(App)` creates an unattached custom view in the application's generational registry.
Its `Tag`, `Bounds`, `Visible`, `Enabled`, `SizeHint`, and `SizePolicy` properties are live registry
state. New views are visible and enabled, have empty bounds at `(0, 0)`, a fixed zero size hint, and a
preferred policy on both axes. `Destroy` invalidates the handle. Destroyed slots may be reused,
always with a new generation, so an old copied handle remains stale and the new view starts with the
default state. Closing the application invalidates every remaining view. `TuiView.Empty(App)` remains
an action-source sentinel and is not a destroyable registry view.

## Headless custom-view lifecycle

`TuiCustomView.Create(App)` creates a typed view backed by the normal view registry. `AsView()`
returns the identity accepted by containers and layouts. `IsAlive()` and `Destroy()` retain the
normal generational-handle behavior.

Attach, detach, resize, focus, blur, close-request, and closed notifications are single-handler
record events:

```pascal
Custom.OnAttach := procedure(Sender: TuiCustomView) begin ... end;
Custom.OnDetach := procedure(Sender: TuiCustomView) begin ... end;
Custom.OnResize := procedure(Sender: TuiCustomView; OldBounds: TuiRect; NewBounds: TuiRect) begin ... end;
Custom.OnFocus := procedure(Sender: TuiCustomView) begin ... end;
Custom.OnBlur := procedure(Sender: TuiCustomView) begin ... end;
```

`OnAttach` runs synchronously after the parent relation becomes visible and before the first layout.
`OnDetach` runs only for a view that was attached. Parent and layout relations have already been
removed, but the sender remains live for the callback; destruction completes afterward.

`OnResize` runs after the application layout phase and before `OnTick`. It is raised only for an
attached custom view whose bounds actually changed. Multiple changes before delivery are coalesced:
`OldBounds` is the first previous rectangle and `NewBounds` is the final rectangle. A bounds change
made inside `OnResize` is delivered during a later iteration rather than recursively.

Lifecycle dispatch revalidates handles after callbacks. Destroying the sender from `OnAttach`,
`OnDetach`, or `OnResize` is safe, and a destroyed view receives no later pending resize event.
Replacing an event handler replaces the previous handler; assigning `nil` clears it.

One custom view per application may be focused. `Focus()` succeeds only when the view is attached
through visible, enabled ancestors to the application's desktop. `Blur()` returns `false` when the
view did not own focus. The read-only `Focused` property observes the current owner. Moving focus
delivers the old view's `OnBlur` before the new view's `OnFocus`; hiding or disabling the focused
view or one of its ancestors blurs it immediately. A focus request made from a focus callback is
queued until the current transition completes, so callbacks do not recurse.

`Close()` is the explicit, vetoable custom-view close path. `OnCloseRequest` returns `true` to
accept or `false` to leave the view attached, live, and focused. An accepted close removes ownership,
blurs the view when needed, delivers `OnDetach`, then invokes `OnClosed` while the sender remains
live for inspection. The handle becomes stale when `Close()` returns. Direct `Destroy()`, container
removal, and application shutdown are ownership teardown operations and do not invoke
`OnCloseRequest` or `OnClosed`.

## Headless custom-view measurement and paint

`TuiCustomView.OnMeasure` is a function event that receives the exact `TuiMeasureSpec` requested by
the layout engine and returns the view's intrinsic `TuiMeasureResult`:

```pascal
Custom.OnMeasure :=
  function(Sender: TuiCustomView; Spec: TuiMeasureSpec): TuiMeasureResult
  begin
    return TuiMeasureResult.Fixed(TuiSize.Create(12, 3))
  end;
```

The existing per-axis `SizePolicy` and measure constraints are applied after the handler returns.
With no assigned handler, measurement uses the view's `SizeHint`. Assigning, replacing, or clearing
`OnMeasure` invalidates an owning layout. Measurement callbacks may run more than once and must be
deterministic and free of application or view-tree mutation.

`TuiCustomView.OnPaint` receives a `TuiCanvas` and `TuiPaintContext` after layout and pending resize
delivery, before `OnTick`:

```pascal
Custom.OnPaint :=
  procedure(Sender: TuiCustomView; Canvas: TuiCanvas; Context: TuiPaintContext)
  begin
    Canvas.FillRect(Context.Bounds, TuiCell.Create(' ', TuiStyleRole.Normal))
  end;
```

`Context.Bounds` starts at `(0, 0)` and has the view's resolved extent. `Context.Clip` is expressed
in the same local coordinates and intersects the application surface, the view, and every visible
ancestor. Canvas operations use local coordinates and cannot write outside that clip. `Enabled`
combines the view and all ancestors; `Focused` reflects the application's current custom-view focus
owner. The context uses the default palette.

`Invalidate()` schedules a later repaint without invoking `OnPaint` recursively. Bounds, visibility,
enabled state, handler replacement, attachment, removal, and application resize also invalidate the
headless frame. A frame repaint starts with blank cells and redraws attached visible custom views;
when nothing is invalid, another iteration does not call `OnPaint`. `TuiApplication.Surface` exposes
the latest retained surface for deterministic inspection. The surface is initialized immediately and
uses the new blank extent immediately after `ResizeForTest`, before the next paint pass.

The headless renderer walks the retained tree from back to front within each container. A complete
child subtree stays together in that ordering, and later siblings paint over earlier siblings.
Terminal presentation and runtime enforcement of callback mutation and canvas lifetime restrictions
are not implemented.

## Headless containers

`TuiContainer.Create(App)` creates a live container backed by a view handle. `Add(Child)` accepts one
live view from the same application and rejects an already attached view or an attachment that would
create an ownership cycle. `Contains(Child)` reports the current direct ownership relation.
`Remove(Child)` removes and destroys that child, so its handle becomes stale. Containers may be
nested: destroying a container's `AsView()` handle or removing an attached container destroys every
descendant depth-first.

Direct children have a stable back-to-front order. A newly added child starts at the front.
`BringToFront(Child)` and `SendToBack(Child)` return `true` when they change that order and invalidate
the next frame; repeating the already satisfied operation returns `false`. Moving a container moves
its complete descendant subtree as one Z-order unit. Layout item order and resolved bounds are not
changed by these operations.

`Layout` is an `option of TuiLayout` property. Assign `Some(Layout)` to attach the one layout owned
by the container. Assigning `None`, assigning a different layout, or destroying the container destroys
the former layout. Destroying an attached layout directly clears the property. A layout cannot belong
to more than one container.

`NeedsLayout()` reports whether the attached layout requires a headless pass. A newly attached
layout is dirty. Child insertion or removal, view visibility, size hints, size policies, layout
settings, stacked-page selection, and invalidation from a nested layout make its root dirty.
Repeated changes before the next pass are coalesced.

`PerformLayout()` arranges a dirty root into `(0, 0, ContainerWidth, ContainerHeight)`, marks the
complete nested layout tree clean, and returns `true`. It also runs when the container width or
height changes. It returns `false` when no layout is attached or the existing geometry remains
current. Moving a container without resizing it does not require a new pass because child bounds
are local to the container.

The read-only `LayoutFit` property measures the current root against the container size. Its
`Minimum` and `Available` fields retain those two extents. `Overflow` contains the positive shortage
on each axis, and `Fits()` is true only when both shortages are zero. A container without a layout
has a zero minimum and always fits. The property reflects current inputs immediately; callers do not
need to run a layout pass first.

`TuiDesktop.Create(App)` creates the one explicit headless root container for an open application.
It exposes the same `Add`, `Contains`, `Remove`, `NeedsLayout`, `PerformLayout`, and read-only
`LayoutFit` surface, and becomes stale when its application closes. No implicit desktop is created.
The desktop starts at the application size, follows `ResizeForTest`, and its dirty root layout is
completed by `Tick` or `RunIterations`.

`HitTest(Point)` accepts zero-based application coordinates and returns the topmost custom view whose
resolved paint clip contains the point. Hidden views, effectively disabled views, clipped portions,
containers, and points outside the application surface do not produce a hit. Hit-testing therefore
cannot select a cell that the same view could not paint.

`FocusNext()` and `FocusPrevious()` traverse attached custom views in retained depth-first child
order, wrap at both ends, and skip hidden or effectively disabled subtrees. With no current focus,
forward traversal starts at the first eligible view and backward traversal at the last. They return
`false` when no eligible custom view exists. Z-order changes update this retained traversal order.

`TuiModal.Open(Container)` limits focus, hit-testing, and routed input to an attached, visible,
enabled container inside the current modal subtree. Opening a modal clears focus and pointer capture
outside the new subtree. `FocusNext` and
`FocusPrevious` then traverse only modal descendants. `Close()` restores the focus that existed
before opening when it remains eligible. Modal roots may be nested; only the top modal may close, so
`Close()` returns `false` for an out-of-order close or an already inactive handle. Destroying a modal
root deactivates its handle automatically.

## Headless scroll views

`TuiScrollView.Create(App)` creates a live viewport backed by an internal `TuiContainer`.
`AsView()` exposes the view used for bounds and parent attachment, while `AsContainer()` exposes the
container identity when direct child ownership is needed. The scroll view owns one root `Layout`
with the same attachment, replacement, and destruction rules as a container.

`ViewportSize` is the current view width and height. `ContentSize` is the component-wise maximum of
that viewport and the root layout's unbounded preferred size. `MaximumOffset` is therefore the
positive content excess on each axis. The read-write `Offset` property rejects negative coordinates
and clamps coordinates beyond that maximum. Content or viewport shrinkage clamps a retained offset
when it is next observed or laid out.

`NeedsLayout()` reports root invalidation or viewport resizing. `PerformLayout()` arranges content at
the negative local origin `(-Offset.X, -Offset.Y)` using the complete content size. This makes the
resolved child bounds deterministic in headless applications. Custom-view descendants are clipped
to the scroll view bounds by the headless paint pass. Scroll input and scroll bars are not part of
this headless control.

Destroying a scroll view destroys its internal container and root layout. Views referenced by the
layout remain governed by the view tree and are not destroyed merely because the layout is destroyed.

## Headless layouts

`TuiLayout.Create(App)` creates a live application-scoped layout handle. Like views, its `Tag`
property is live registry state, `Destroy` invalidates it, and a reused slot always has a new
generation. Closing the application invalidates all remaining layouts. `TuiLayoutItems` manages an
ordered item list for each live layout.

`TuiLayout.CreateKind(App, Kind)` creates a common handle with an explicit immutable kind;
`Create(App)` is its horizontal shorthand. Applications normally use the typed constructors below.

`TuiHorizontalLayout.Create(App)`, `TuiVerticalLayout.Create(App)`, `TuiGridLayout.Create(App)`,
`TuiFormLayout.Create(App)`, and `TuiStackedLayout.Create(App)` create typed handles. `AsLayout()`
returns the common identity accepted by settings, measurement, arrangement, nesting, and container
operations. `IsAlive()` and `Destroy()` retain the normal generational-handle behavior.

The common layout's read-only `Kind` property identifies its horizontal, vertical, grid, form, or
stacked family.
`TuiLayoutSettings` retains non-negative margins and non-negative spacing. A generic `TuiLayout`
defaults to horizontal kind with zero margins and spacing. The settings belong to the live registry
generation and are discarded when the layout is destroyed.

## Measurement values

`TuiMeasureConstraint.Unbounded()` has no upper limit. `TuiMeasureConstraint.AtMost(Limit)` accepts a
non-negative upper limit. `TuiMeasureSpec.Create(Width, Height)` combines one constraint for each
axis; `Unbounded()` and `AtMost(TuiSize)` construct common specifications.

`TuiMeasureResult.Create(Minimum, Preferred, Maximum)` requires the three sizes to be ordered on both
axes: minimum no greater than preferred, and preferred no greater than maximum. `Fixed(Size)` creates
a result with the same value for every size.

`TuiLayoutMeasure.Measure(Layout, Spec)` recursively combines visible view size hints, nested layout
results, spacers, spacing, and margins. `AtMost` constrains preferred and maximum extents without
reducing a declared minimum. Hidden views do not participate. `MeasureItem` exposes the same policy
evaluation for one layout item and is primarily used by the allocator.

## Size policies

`TuiSizePolicyKind` has `Fixed`, `Minimum`, `Maximum`, `Preferred`, and `Expanding` values. A
`TuiSizePolicy` contains independent `Horizontal` and `Vertical` fields. Use `Create` for mixed axes
or one of the uniform constructors: `Fixed()`, `Minimum()`, `Maximum()`, `Preferred()`, and
`Expanding()`. A view's `SizePolicy` property retains the selected value. Measurement applies the
policy on each axis, and arrangement uses `Expanding` to request remaining main-axis cells.

## Margins and alignment

`TuiMargins.Create(Left, Top, Right, Bottom)` requires non-negative cell counts. `Uniform(Value)`
uses one value on all sides, while `Symmetric(Horizontal, Vertical)` uses one value per axis.
`Horizontal()` and `Vertical()` return the combined margins on each axis.

`TuiAlignmentKind` has `Leading`, `Center`, `Trailing`, and `Fill` values. Leading means left on the
horizontal axis and top on the vertical axis; trailing means right and bottom. `TuiAlignment.Create`
combines independent axes, and matching uniform constructors create the common cases. Arrangement
reserves margins and spacing, then applies alignment inside each allocated slot.

## Layout items

`TuiSpacer.Fixed(Extent)` describes an exact empty extent on a layout's main axis.
`TuiSpacer.Expanding(MinimumExtent)` describes empty space that may receive additional cells. Both
constructors reject negative extents.

A `TuiLayoutItem` describes exactly one live view, one live nested layout, or one spacer. Use
`ForView`, `ForLayout`, or `ForSpacer`; each constructor also receives an alignment and a
non-negative stretch factor. View and layout constructors reject stale handles, and an empty action
source is not a view item.

`TuiLayoutItems.Add(Layout, Item)` appends an item and returns `true`. `Count` and `Get` expose the
current stable order. `RemoveAt` removes one item, while `Clear` removes all items; both return `true`
after a successful mutation. Indexes are zero-based. `CanAdd` reports whether an already constructed
item currently satisfies the live ownership contract without mutating either layout.

A live view may occur in only one layout item list. Removing or clearing its item does not destroy the
view because the view tree owns views. A nested layout has exactly one owner: either a container or
another layout. Removing or clearing a nested item destroys that nested layout and its nested
descendants. Destroying a parent layout does the same. Directly destroying an attached view removes
its item; directly destroying a nested layout removes its item. Cycles and cross-application items are
rejected.

Grid layouts use `TuiGridItems` instead of adding directly through `TuiLayoutItems`.
`TuiGridPlacement.Cell(Row, Column)` creates a single cell; `Create(Row, Column, RowSpan,
ColumnSpan)` creates a spanning placement. Indexes are zero-based, spans must be positive, and
placements may not overlap. `TuiGridItems.Add`, `Count`, `Get`, `PlacementAt`, `RemoveAt`, and
`Clear` retain the same view and nested-layout ownership rules as the common list. A one-dimensional
`TuiSpacer` is not a valid grid item.

Form layouts are specialized two-column grids. `TuiFormItems.AddRow(Form, LabelView, FieldView)`
adds a label aligned trailing and centered beside a fill-aligned field. `AddRowItems` accepts two
explicit view items when different alignment or stretch values are needed. `Count`, `LabelAt`,
`FieldAt`, `RemoveAt`, and `Clear` address zero-based rows. Removing a row detaches both views but
does not destroy them. Destroying either view directly removes the complete row and detaches the
remaining view.

Stacked layouts use `TuiStackedItems.Add`, `Count`, `Get`, `RemoveAt`, and `Clear` for view or nested
layout pages; spacers are rejected. `CurrentIndex` is a read-write zero-based property. It is `-1`
for an empty stack, selects the first page after the first insertion, and is clamped after page
removal. Assigning any index outside the current page range is rejected.

## Headless layout allocation

`TuiLayoutArrange.Arrange(Layout, Bounds)` performs a complete recursive layout pass for box, grid,
form, and stacked layouts. It reserves margins and inter-item spacing, assigns minimum sizes, moves items toward
preferred sizes, and then distributes remaining cells by stretch weight or expanding policy.
Indivisible cells go to earlier items or tracks in stable order. Stretch enlarges an allocated slot;
the item inside that slot still respects its finite maximum size and alignment.

`Leading`, `Center`, `Trailing`, and `Fill` align the final item rectangle on each axis. Fixed and
expanding spacers consume main-axis space but do not receive view bounds. Hidden views keep their
previous bounds and are excluded from the pass. When the supplied bounds are below the combined
minimum, items keep their minimum geometry and can extend beyond the supplied rectangle; a parent
container clips custom-view painting of that overflow. `Container.LayoutFit` reports the exact
shortage without reducing item bounds. Headless layout therefore detects terminal-too-small state
without altering the minimum geometry.

```pascal
var Row: TuiHorizontalLayout := TuiHorizontalLayout.Create(App);
TuiLayoutSettings.SetMargins(Row.AsLayout(), TuiMargins.Uniform(1));
TuiLayoutSettings.SetSpacing(Row.AsLayout(), 1);
TuiLayoutItems.Add(Row.AsLayout(), TuiLayoutItem.ForView(Left, TuiAlignment.Fill(), 1));
TuiLayoutItems.Add(Row.AsLayout(), TuiLayoutItem.ForView(Right, TuiAlignment.Fill(), 1));
TuiLayoutArrange.Arrange(Row.AsLayout(), TuiRect.Create(0, 0, 40, 10));
```

A grid infers its rows and columns from its visible placements. Spanning items contribute their size
requirements across every covered track, while fixed spacing is counted once between adjacent
tracks. Item stretch provides the relative growth weight for every covered row and column.

A form uses the same two-axis measurement and allocation rules with one inferred row per label-field
pair. A stack measures the component-wise maximum of every visible page, so changing
`CurrentIndex` does not resize its parent. Arrangement assigns bounds only to the selected visible
page; other pages retain their previous bounds until selected.

## Commands and actions

`TuiCommand.Create(Value)` creates an application command and requires `Value >= 1024`. Values
`1..1023` are reserved for the standard library and can be created internally with
`TuiCommand.FromStandard`. Zero and negative identities are invalid.

```pascal
var Save: TuiAction := TuiAction.Create(App, TuiCommand.Create(1024), 'Save');
Save.Enabled := true;
Save.OnExecute :=
  procedure(Sender: TuiAction; Source: TuiView)
  begin
    SaveDocument()
  end;
```

An action provides live `Command`, `Text`, `Enabled`, `Visible`, `Checked`, and optional `Shortcut`
properties. `Activate(Source)` invokes `OnExecute` synchronously when enabled and returns whether
activation was accepted. The source must belong to the same application. `Destroy` invalidates the
action and releases its handler.

Assign a `Some(TuiKeyEvent)` shortcut to activate an action from injected input. Key matching compares
the key kind, character, and every modifier exactly. After an unconsumed focused custom-view key
handler, the router checks shortcuts before `App.Input.OnKey`. When multiple enabled actions use the
same shortcut, the first created action wins; disabled actions do not consume it. Shortcut activation
uses `TuiViewKind.Empty` as its source.

## Buttons and direct events

`TuiButton.Create(App, Text)` creates a headless retained button. `AsView` returns its live,
attachable view identity; `Text`, `Enabled`, and `Action` use registry state; `Focused` reports the
application focus owner; and `OnClick` is a single-handler event.

```pascal
var Button: TuiButton := TuiButton.Create(App, 'Save');
Button.Action := Save;
Button.OnClick := procedure(Sender: TuiButton) begin ... end;
Desktop.Add(TuiButton.AsView(Button));
Button.Click()
```

`Click` applies a deterministic order:

1. reject the click when the button is disabled;
2. activate the bound action when present;
3. revalidate the button;
4. invoke `OnClick` when the button still exists.

If the action destroys the button or closes its application, `OnClick` is skipped. `Focus()` requests
focus for an attached eligible button. A focused button accepts `Enter` and `Space`; a left pointer
down inside its retained bounds focuses and activates it. Both paths call `Click`, preserving the
action-before-`OnClick` order and supplying `TuiViewKind.Button` as the action source. Disabled
buttons are excluded from focus and hit testing, so their input falls through to the application
fallback.

Attached visible buttons paint their bounded local text as `[Text]`. Normal buttons use the
`Normal` role, focused buttons use `Focused`, and effectively disabled buttons use `Disabled`.
The remaining cells in the resolved button bounds are cleared with that same role. Text changes,
enabled-state changes, and focus transitions invalidate the next headless frame; button paint
participates in the same retained tree order as custom views.

A button's intrinsic measurement is one cell high and its label's grapheme width plus two cells for
the brackets. This result is fixed on both axes. Replacing `Text` invalidates every owning layout,
so the next headless layout pass uses the new intrinsic width.

## Labels

`TuiLabel.Create(App, Text)` creates a non-focusable retained text label. `AsView` returns its live,
attachable view identity and `Text` is a writable registry-backed property.

```pascal
var Label: TuiLabel := TuiLabel.Create(App, 'Name');
Desktop.Add(TuiLabel.AsView(Label));
```

An attached visible label paints its text at the local origin. It uses the `Normal` role, or the
`Disabled` role when the label or an ancestor is disabled. Its fixed intrinsic size is one cell high
and the grapheme display width of `Text`. Replacing `Text` invalidates both its paint and every
owning layout, so the next layout pass uses the new width. `Destroy` releases the backing view and
makes the label handle stale.

## Input lines

`TuiInputLine.Create(App, MaxLength)` creates a focusable retained one-line text field. `MaxLength`
must be positive and is its fixed width in cells. The read-only `MaxLength`, read-write `Text`, and
read-only `Focused` properties validate the live handle.

```pascal
var Name: TuiInputLine := TuiInputLine.Create(App, 20);
Desktop.Add(TuiInputLine.AsView(Name));
Name.Focus()
```

Focused input lines consume plain character and Space keys, plus Left, Right, Home, Backspace, and
Delete. A left pointer-down focuses the field and places its character cursor. Input beyond
`MaxLength` is not accepted; assigning an overlong `Text` value is rejected. Attached visible
input lines paint their text at the local origin with the `Normal`, `Focused`, or `Disabled` role.
`Destroy` releases the backing view and makes the handle stale.

## Implementation (contributors)

`Std.Tui2` is a source-level standard-library facade in [`lib/Std/Tui2.fpas`](../../../../lib/Std/Tui2.fpas).
Geometry and cell values live in focused private units under `lib/Std/Tui2/Geometry/` and
`lib/Std/Tui2/Cells/`. Live application, input, action, view, layout, and button concerns are
separated under `Runtime/`, `Input/`, `Actions/`, `Views/`, `Layouts/`, and `Controls/`. The facade is exported by
[`lib/stdlib.fpasprj`](../../../../lib/stdlib.fpasprj).

## See also

- [Standard library reference](../README.md)
- [Record events](../../language/types/record-events.md)
- [Record properties](../../language/types/record-properties.md)
- [Units](../../program-structure/units.md)
