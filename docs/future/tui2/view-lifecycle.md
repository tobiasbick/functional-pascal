# Std.Tui2 view lifecycle

Std.Tui2 defines a deterministic lifecycle for applications and views. Built-in controls implement the lifecycle internally. Application-provided hooks are primarily intended for `TuiCustomView` and application lifecycle boundaries.

## Explicit construction instead of OnInit

Std.Tui2 does not define a general `OnInit` callback. Construction and configuration happen explicitly before a view is attached:

```pascal
var View: TuiCustomView := TuiCustomView.Create(App);
var Base: TuiView := TuiCustomView.AsView(View);
Base.SizePolicy := Policy;
View.OnPaint := PaintContent;
Parent.Add(Base)
```

Attaching the configured view starts its live tree lifecycle.

## Application lifecycle

| Hook | Timing | Return |
| --- | --- | --- |
| `OnStart` | After terminal setup and desktop creation, before the first layout and paint. | none |
| `OnStop` | After the run loop stops, before terminal restoration and handle invalidation. | none |
| `OnTick` | Once per application iteration when ticking is enabled. | none |

`OnStart` and `OnStop` name observable run-loop boundaries. They do not replace explicit application construction or close operations.

Implemented headless delivery: `RunIterations` invokes `OnStart` once before its first iteration.
Budget exhaustion leaves the application started and open for another bounded run. `Quit` invokes
`OnStop` once at orderly close; a posted quit request is honored before `OnTick`.

## View lifecycle hooks

| Hook | Timing | Return |
| --- | --- | --- |
| `OnAttach` | After insertion into a live parent and before first measurement. | none |
| `OnDetach` | After removal from routing and layout, before ownership is released. | none |
| `OnMeasure` | When the layout engine needs size information. | `TuiMeasureResult` |
| `OnResize` | After resolved bounds change, before the next paint. | none |
| `OnPaint` | When an invalid region intersects the view. | none |
| `OnFocus` | After the view becomes focused. | none |
| `OnBlur` | After the view loses focus. | none |
| `OnCloseRequest` | Before a closeable view is removed. | `boolean` |
| `OnClosed` | After closing completed and the view no longer participates in dispatch. | none |

Implemented headless transitions: `TuiCustomView.OnAttach` runs after its parent relation is visible.
`OnDetach` runs after parent and layout cleanup while the sender remains live. `OnResize` receives old
and new bounds after layout and before `OnTick`; multiple pending changes coalesce, and changes from
inside the handler wait for a later iteration. Dispatch revalidates handles after each callback.
`OnMeasure` supplies intrinsic custom-view sizing to the existing measure-policy pipeline. `OnPaint`
receives local bounds and an ancestor-clipped canvas during the headless paint phase. One attached,
visible, and effectively enabled custom view may own application focus. Focus changes deliver
`OnBlur` before `OnFocus`, and callback-initiated focus requests are deferred until the current
transition completes. `Close` implements vetoable close-request and ordered closed delivery. Runtime
enforcement of the measurement and paint mutation rules remains open.

For `OnCloseRequest`, `true` allows closing and `false` cancels it. This meaning differs intentionally from raw input handlers, where `true` means consumed.

## Typical sequence

The normal sequence for a custom view is:

```text
construct and configure
attach
measure one or more times
resize when resolved bounds changed
paint when invalid
focus / blur zero or more times
close request when applicable
detach
closed when closing completed
```

Measurement and painting may run many times. No application may assume a fixed count.

## Measurement contract

`OnMeasure` receives a `TuiMeasureSpec` and computes a `TuiMeasureResult` from immutable inputs such as content, available size, and current style.

- It must not mutate the view tree, focus, actions, or application state.
- Repeating it with the same inputs must produce the same result.
- It must not draw terminal cells.
- It may be skipped when cached inputs remain unchanged.
- Width-dependent height is expressed by measuring with an at-most width constraint; the cache key includes the complete measure specification.

Built-in controls provide their own measurement behavior. Applications register `OnMeasure` only for custom views.

## Paint contract

`OnPaint` receives a transient `TuiCanvas` and `TuiPaintContext` describing local bounds, clip, focus, enabled state, and palette.

- Coordinates are local to the view.
- Drawing is clipped to the supplied paint region.
- The handler may draw but must not change layout, ownership, focus, or modal state.
- The canvas is valid only for the duration of the callback.
- Painting reads already resolved bounds; it never initiates layout.
- Invalidating a view during paint schedules a later paint and does not recurse immediately.

Violations produce a clear runtime diagnostic rather than silently corrupting the view tree.

## Resize and focus notifications

`OnResize` runs only when resolved bounds actually change. Layout has completed before it runs, and paint follows afterward if needed.

`OnFocus` and `OnBlur` describe completed focus transitions. They are notifications, not veto points. Focus eligibility is decided before the transition by view state and modal routing.

The headless implementation currently derives eligibility from attachment to the desktop plus the
view's complete visible and enabled ancestor chain. Modal routing and traversal policy remain part of
the input-routing work.

## Close lifecycle

Only closeable views receive `OnCloseRequest`. A rejected request leaves the view attached and focused state unchanged when possible.

Once closing is accepted:

1. remove the view subtree from event routing;
2. release pointer capture and repair focus;
3. detach the subtree;
4. invoke `OnClosed`;
5. invalidate affected handles according to the ownership contract.

The current custom-view order is close request, ownership removal, blur when focused, detach, closed,
and handle invalidation. `OnClosed` receives a live sender for inspection. Direct destruction,
container removal, and application stop do not invoke the vetoable close path.

An application stop is not required to ask every child for permission. Application-specific unsaved-work behavior belongs in an application action or top-level close handler.

## Custom-view event surface

Lifecycle handlers use Pascal-style event assignment:

```pascal
View.OnAttach := AttachHandler;
View.OnDetach := DetachHandler;
View.OnMeasure := MeasureHandler;
View.OnResize := ResizeHandler;
View.OnPaint := PaintHandler;
View.OnFocus := FocusHandler;
View.OnBlur := BlurHandler;
View.OnCloseRequest := CloseRequestHandler;
View.OnClosed := ClosedHandler;
```

Each event has at most one handler. Assignment replaces the previous handler and assignment of
`nil` clears it. A handler may be a named routine, bound record method, or capturing closure. See
[events-and-actions.md](events-and-actions.md).

## Mutation and re-entry

- Notification handlers may update application and view state unless their specific contract forbids it.
- Mutations invalidate layout or paint and are applied after the current callback returns.
- The dispatcher revalidates handles before continuing after a callback.
- Lifecycle callbacks do not run recursively for the same view.
- Nested application runs and uncontrolled callback re-entry are not supported initially.

`OnStop` runs only during orderly shutdown. A panic or fatal runtime error bypasses user callbacks and uses the terminal safety cleanup in [runtime-boundary.md](runtime-boundary.md).
