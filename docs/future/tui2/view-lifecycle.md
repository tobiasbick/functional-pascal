# Std.Tui2 view lifecycle

Std.Tui2 defines a deterministic lifecycle for applications and views. Built-in controls implement the lifecycle internally. Application-provided hooks are primarily intended for `TuiCustomView` and application lifecycle boundaries.

## Explicit construction instead of OnInit

Std.Tui2 does not define a general `OnInit` callback. Construction and configuration happen explicitly before a view is attached:

```pascal
var View: TuiCustomView := TuiCustomView.New(App);
TuiView.SetSizePolicy(TuiCustomView.AsView(View), Policy);
TuiCustomView.SetPaintHandler(View, PaintContent);
TuiContainer.Add(Parent, TuiCustomView.AsView(View))
```

Attaching the configured view starts its live tree lifecycle.

## Application lifecycle

| Hook | Timing | Return |
| --- | --- | --- |
| `OnStart` | After terminal setup and desktop creation, before the first layout and paint. | none |
| `OnStop` | After the run loop stops, before terminal restoration and handle invalidation. | none |
| `OnTick` | Once per application iteration when ticking is enabled. | none |

`OnStart` and `OnStop` name observable run-loop boundaries. They do not replace explicit application construction or close operations.

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

## Close lifecycle

Only closeable views receive `OnCloseRequest`. A rejected request leaves the view attached and focused state unchanged when possible.

Once closing is accepted:

1. remove the view subtree from event routing;
2. release pointer capture and repair focus;
3. detach the subtree;
4. invoke `OnClosed`;
5. invalidate affected handles according to the ownership contract.

An application stop is not required to ask every child for permission. Application-specific unsaved-work behavior belongs in an application action or top-level close handler.

## Custom-view API shape

The rough registration surface is:

```pascal
TuiCustomView.New(App: TuiApplication): TuiCustomView
TuiCustomView.SetAttachHandler(View: TuiCustomView; Handler: TuiAttachHandler)
TuiCustomView.SetDetachHandler(View: TuiCustomView; Handler: TuiDetachHandler)
TuiCustomView.SetMeasureHandler(View: TuiCustomView; Handler: TuiMeasureHandler)
TuiCustomView.SetResizeHandler(View: TuiCustomView; Handler: TuiResizeHandler)
TuiCustomView.SetPaintHandler(View: TuiCustomView; Handler: TuiPaintHandler)
TuiCustomView.SetFocusHandler(View: TuiCustomView; Handler: TuiFocusHandler)
TuiCustomView.SetBlurHandler(View: TuiCustomView; Handler: TuiBlurHandler)
TuiCustomView.SetCloseRequestHandler(View: TuiCustomView; Handler: TuiCloseRequestHandler)
TuiCustomView.SetClosedHandler(View: TuiCustomView; Handler: TuiClosedHandler)
```

Each hook has at most one handler. Setting a handler replaces the previous one; a clear operation removes it. The initial handler shapes are part of the API surface and use named FPAS routines.

## Mutation and re-entry

- Notification handlers may update application and view state unless their specific contract forbids it.
- Mutations invalidate layout or paint and are applied after the current callback returns.
- The dispatcher revalidates handles before continuing after a callback.
- Lifecycle callbacks do not run recursively for the same view.
- Nested application runs and uncontrolled callback re-entry are not supported initially.

`OnStop` runs only during orderly shutdown. A panic or fatal runtime error bypasses user callbacks and uses the terminal safety cleanup in [runtime-boundary.md](runtime-boundary.md).
