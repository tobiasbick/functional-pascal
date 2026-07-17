# Std.Tui2

`Std.Tui2` provides terminal UI value types plus the first headless live-object surface for
applications, actions, and buttons. Rendering, layout, terminal acquisition, and interactive input
are not exposed yet.

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
| `TuiCommand` | A validated positive command identity. |
| `TuiView` | A headless application-scoped view handle and action source identity. |
| `TuiContainer` | A headless owner of directly attached views. |
| `TuiDesktop` | The explicit headless root container for one application. |
| `TuiAction` | A reusable operation with live properties and one `OnExecute` event. |
| `TuiButton` | A headless semantic button with an optional action and one `OnClick` event. |
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
modes. `Start`, `Tick`, and `Close` expose deterministic lifecycle boundaries for headless code.
`Close` is idempotent and invalidates every action and button owned by the application.

```pascal
var App: TuiApplication := TuiApplication.OpenForTest(TuiSize.Create(80, 25));
App.OnStart := procedure(Sender: TuiApplication) begin ... end;
App.OnTick := procedure(Sender: TuiApplication; DeltaMilliseconds: integer) begin ... end;
App.OnStop := procedure(Sender: TuiApplication) begin ... end;
App.Start();
App.Tick(16);
App.Close()
```

The lifecycle members are single-handler [record events](../../language/types/record-events.md).
Assigning another compatible handler replaces the previous handler; assigning `nil` clears it.
Copied application handles resolve the same registry state. `Tag` is a read-write live property.

## Headless views

`TuiView.Create(App)` creates an unattached custom view in the application's generational registry.
Its `Tag` property is live registry state; `Destroy` invalidates the handle. Destroyed slots may be
reused, always with a new generation, so an old copied handle remains stale. Closing the application
invalidates every remaining view. `TuiView.Empty(App)` remains an action-source sentinel and is not a
destroyable registry view.

## Headless containers

`TuiContainer.Create(App)` creates a live container backed by a view handle. `Add(Child)` accepts one
live view from the same application and rejects an already attached view. `Contains(Child)` reports
the current direct ownership relation. `Remove(Child)` removes and destroys that child, so its handle
becomes stale. Nested container subtrees are not exposed yet.

`TuiDesktop.Create(App)` creates the one explicit headless root container for an open application.
It exposes the same `Add`, `Contains`, and `Remove` operations, and becomes stale when its application
closes. The eventual `App.Desktop` property is not exposed yet.

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

An action provides live `Command`, `Text`, `Enabled`, `Visible`, and `Checked` properties.
`Activate(Source)` invokes `OnExecute` synchronously when enabled and returns whether activation was
accepted. The source must belong to the same application. `Destroy` invalidates the action and
releases its handler.

## Buttons and direct events

`TuiButton.Create(App, Text)` creates a headless semantic button. Its `Text`, `Enabled`, and `Action`
properties use registry state; `OnClick` is a single-handler event.

```pascal
var Button: TuiButton := TuiButton.Create(App, 'Save');
Button.Action := Save;
Button.OnClick := procedure(Sender: TuiButton) begin ... end;
Button.Click()
```

`Click` applies a deterministic order:

1. reject the click when the button is disabled;
2. activate the bound action when present;
3. revalidate the button;
4. invoke `OnClick` when the button still exists.

If the action destroys the button or closes its application, `OnClick` is skipped. `AsView` returns
the source identity supplied to the action. This API models semantic activation only; keyboard,
mouse, painting, and layout behavior are not implemented by the current button.

## Implementation (contributors)

`Std.Tui2` is a source-level standard-library facade in [`lib/Std/Tui2.fpas`](../../../../lib/Std/Tui2.fpas).
Geometry and cell values live in focused private units under `lib/Std/Tui2/Geometry/` and
`lib/Std/Tui2/Cells/`. Live application, action, view, and button concerns are separated under
`Runtime/`, `Actions/`, `Views/`, and `Controls/`. The facade is exported by
[`lib/stdlib.fpasprj`](../../../../lib/stdlib.fpasprj).

## See also

- [Standard library reference](../README.md)
- [Record events](../../language/types/record-events.md)
- [Record properties](../../language/types/record-properties.md)
- [Units](../../program-structure/units.md)
