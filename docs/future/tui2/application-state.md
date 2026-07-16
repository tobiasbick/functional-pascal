# Std.Tui2 application state and handler signatures

FPAS passes named routines as first-class callbacks but does not provide capturing closures. Std.Tui2 therefore does not attempt to store arbitrary type-erased application objects inside TUI handles.

## Application-owned unit state

The initial application pattern is a private mutable record in the application's own unit:

```pascal
unit MyApp.Main;

private mutable var State: AppState := DefaultState();

procedure Save(App: TuiApplication; Action: TuiAction; Source: TuiView);
begin
  SaveDocument(State.Document)
end;
```

This is explicit module state, not a hidden TUI global. Std.Tui2 owns UI state; the application unit owns domain state.

Application unit state is accessed on the main task only. Worker tasks receive immutable input values and return results through `TuiApplication.Post`; they do not mutate the application record directly.

`TuiApplication.Tag` and `TuiView.Tag` provide an integer association key, defaulting to zero. Tags identify application records or domain entities but are not a heterogeneous data store.

## Fixed callback shapes

The initial handler contracts are:

```pascal
TuiApplicationHandler = procedure(App: TuiApplication)

TuiTickHandler = procedure(
  App: TuiApplication;
  DeltaMilliseconds: integer
)

TuiActionHandler = procedure(
  App: TuiApplication;
  Action: TuiAction;
  Source: TuiView
)

TuiTextChangedHandler = procedure(
  App: TuiApplication;
  Source: TuiInputLine;
  Value: string;
  Origin: TuiChangeOrigin
)

TuiCheckedChangedHandler = procedure(
  App: TuiApplication;
  Source: TuiCheckBox;
  Value: boolean;
  Origin: TuiChangeOrigin
)

TuiListSelectionChangedHandler = procedure(
  App: TuiApplication;
  Source: TuiListBox;
  Selected: integer;
  Origin: TuiChangeOrigin
)

TuiRadioSelectionChangedHandler = procedure(
  App: TuiApplication;
  Source: TuiRadioGroup;
  Selected: integer;
  Origin: TuiChangeOrigin
)

TuiKeyHandler = function(
  App: TuiApplication;
  Source: TuiView;
  Key: TuiKey
): boolean

TuiMouseHandler = function(
  App: TuiApplication;
  Source: TuiView;
  Event: TuiEvent
): boolean

TuiAttachHandler = procedure(App: TuiApplication; View: TuiCustomView)
TuiDetachHandler = procedure(App: TuiApplication; View: TuiCustomView)

TuiMeasureHandler = function(
  App: TuiApplication;
  View: TuiCustomView;
  Spec: TuiMeasureSpec
): TuiMeasureResult

TuiResizeHandler = procedure(
  App: TuiApplication;
  View: TuiCustomView;
  Bounds: TuiRect
)

TuiPaintHandler = procedure(
  App: TuiApplication;
  View: TuiCustomView;
  Canvas: TuiCanvas;
  Context: TuiPaintContext
)

TuiFocusHandler = procedure(App: TuiApplication; View: TuiCustomView)
TuiBlurHandler = procedure(App: TuiApplication; View: TuiCustomView)

TuiCloseRequestHandler = function(
  App: TuiApplication;
  View: TuiCustomView
): boolean

TuiClosedHandler = procedure(App: TuiApplication; View: TuiCustomView)
```

For `TuiCloseRequestHandler`, `true` allows closing. For raw key and mouse handlers, `true` consumes the event.

Measure and paint handlers also obey the purity and mutation rules in [view-lifecycle.md](view-lifecycle.md).

Each handler slot stores one named routine. There is no implicit receiver, closure capture, or multicast subscriber list.

## Multiple applications

Only one application may own the interactive terminal at a time. Sequential application instances are supported. Application code that needs separate state instances stores them in its own unit and uses `TuiApplication.Tag` as the lookup key.

If FPAS later gains capturing closures or bound record methods, they may satisfy these same function types without changing the TUI event model.
