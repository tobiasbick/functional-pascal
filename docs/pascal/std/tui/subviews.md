# Cached subviews

`TuiSubview of T` reuses an immutable element tree while its explicitly selected
dependencies remain equivalent. Import it through `Std.Ui.Terminal`.

Create a subview once, outside `View`, with:

```text
TuiSubview.Create(
  Build: function(Dependencies: T): TuiElement;
  Equal: function(Previous: T; Current: T): boolean
): TuiSubview of T
```

Inside `View`, call `Subview.Render(Dependencies)`. The argument is the selected
part of the model; a record can combine several inputs. `Build` runs on the
first use and when `Equal` returns false. Other model changes leave that
subview's construction untouched. An explicit comparison also supports
dependency types that have no built-in equality.

```pascal id=cached-header run
program CachedHeader;

uses Std.Ui.Terminal as Tui;

type Model = record
  Title: string;
  Count: integer;
end record;

function BuildHeader(Title: string): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel(Title);
end function;

function EqualTitle(Previous: string; Current: string): boolean;
begin
  return Previous = Current;
end function;

const Header: Tui.TuiSubview of string := Tui.TuiSubview.Create(BuildHeader, EqualTitle);

function View(State: Model): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeColumn([
    Header.Render(State.Title),
    Tui.TuiElementBuilders.MakeLabel('%d'.Format(State.Count))
  ]);
end function;

function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;
begin
  return Model(Title := State.Title, Count := State.Count + 1);
end function;

begin
  const App := Tui.TuiApplication.OpenForTest(Tui.TuiSize.Create(20, 4));
  discard App.RunIterations(Model(Title := 'Counter', Count := 0), Update, View, 3, 0);
  App.Close();
end.
```

Here the counter changes on every message, while the header is constructed once.
`View` still runs on each render request. Create the subview outside that callback;
creating a new instance on every call defeats reuse.

## Dependency contract

`Build` and `Equal` must depend only on their arguments. Include every input
that affects the returned tree: content, identities, actions, focus, controlled
editing state, menu/modal state and any model-selected style or geometry.
Equal dependencies must produce interchangeable trees, including routing data.
Do not use these callbacks for I/O, message delivery, resource management or
other work that must happen on every update. FPAS does not enforce purity.

Dependencies have ordinary FPAS value semantics. A copied handle or closure
does not snapshot its external mutable state. Select the relevant immutable
values instead, or include a revision in the dependency record and compare it
explicitly. There is no automatic dependency tracking or model subscription.

## Ownership and rendering

- Rendering is synchronous on the application's serial host task and is only
  allowed inside `View`. Callbacks must not reenter the same subview, rerun
  their application or close it.
- Copies of a subview share its cache. Each live application has its own entry,
  even when applications share one subview definition.
- Each entry retains only its most recent dependency value and tree. If
  `Render` is not called during a completed frame, the host drops that entry.
  Closing the host drops all its entries, without requiring application cleanup.
  A live tree still owns its elements, and a live subview still owns its builder.
- Nested subviews are ordinary composition. If a parent reuses its complete
  tree, its builder does not call its children. Those children's independent
  entries expire; their elements remain alive through the parent's tree.
- Cached elements undergo the same tree validation, layout, painting and input
  routing as other elements. Invalid IDs and duplicate control IDs still fail.
- Resize recomputes host geometry without requiring a new dependency value.
  Pressed-button state and palette presentation remain host responsibilities.
  Model-dependent layout, focus, menus and modal changes belong in dependencies.

General application stores and subscriptions are separate from subview caching.
Runnable applications include [Notes](../../../../apps/notes/README.md),
[Local Chat](../../../../apps/local-chat/README.md),
[fpastris](../../../../apps/fpastris/README.md) and the
[Mandelbrot explorer](../../../../examples/math/mandelbrot/README.md).
See [application execution](application.md#headless-frame-and-routing-order)
and [immutable elements](elements.md).
