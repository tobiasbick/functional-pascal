# Std.Tui3 API surface

This inventory is a planning sketch, not a frozen specification. Implemented symbols will
be documented under `docs/pascal/std/tui3/` only after they exist.

## Type categories

### Value records

| Type | Purpose |
| --- | --- |
| `TuiPoint` / `TuiSize` / `TuiRect` | Geometry. |
| `TuiColor` / `TuiStyle` / `TuiStyleRole` / `TuiCell` / `TuiPalette` | Painting values. |
| `TuiSurfaceSnapshot` | Explicit immutable copy of the last painted surface for inspection. |
| `TuiSizePolicy` / `TuiMargins` / `TuiAlignment` / `TuiSpacer` | Layout inputs. |
| `TuiMeasureSpec` / `TuiMeasureResult` / `TuiLayoutFit` | Measurement results. |
| `TuiKeyEvent` | Alias of `Std.Console.KeyEvent`. |
| `TuiPointerEvent` | Normalized pointer action, button, position, modifiers. |
| `TuiControlId` | Positive identity unique among interactive nodes in one tree. |
| `TuiAction` | Positive application-intent identity; repetitions are allowed. |
| `TuiMsg` | Framework message delivered to `Update`. |
| `TuiCmd` | Closed runtime-control command returned by `Update`. |
| `TuiElement` | Immutable UI node description. |
| `TuiMenuItem` / `TuiStatusItem` | Chrome descriptions. |

These records do not own live widgets.

### Application host

| Type | Purpose |
| --- | --- |
| `TuiApplication` | Owns the run loop, headless surface, and terminal session when interactive. |

`TuiApplication` is a host, not a view registry for application widgets. Its working surface and
message queue are mutable private resources. A clipped canvas is an internal frame-scoped
capability, not a large public cell-grid value.

## Core operations

### MVU

```pascal
TuiApplication.OpenForTest(Size: TuiSize): TuiApplication
TuiApplication.Inject(App; Msg: TuiMsg)
TuiApplication.RunIterations<TModel>(
  App: TuiApplication;
  InitialModel: TModel;
  Update: function(State: TModel; Msg: TuiMsg; mutable Cmd: TuiCmd): TModel;
  View: function(State: TModel): TuiElement;
  IterationCount: integer;
  DeltaMilliseconds: integer
): TModel
TuiApplication.SurfaceSnapshot(App): TuiSurfaceSnapshot
TuiApplication.Close(App)

TuiApplication.Run<TModel>(
  InitialModel: TModel;
  Update: function(State: TModel; Msg: TuiMsg; mutable Cmd: TuiCmd): TModel;
  View: function(State: TModel): TuiElement
): TModel
```

The runtime initializes `Cmd := TuiCmd.None` before every `Update` call. This shape uses a generic
routine and a mutable output parameter because FPAS has no tuple/product return syntax and no
generic record type for `TuiUpdateResult<TModel>`. Phase 0 must compile this exact pattern before the
API is treated as viable.

### Element constructors (illustrative)

```pascal
Tui.None: TuiElement
Tui.Desktop(Focused: option of TuiControlId; Children: array of TuiElement): TuiElement
Tui.Window(Title: string; Children: array of TuiElement): TuiElement
Tui.Dialog(Title: string; Children: array of TuiElement): TuiElement
Tui.MenuBar(Items: array of TuiMenuItem): TuiElement
Tui.StatusLine(Text: string): TuiElement
Tui.Row(Children: array of TuiElement): TuiElement
Tui.Column(Children: array of TuiElement): TuiElement
Tui.Grid(...): TuiElement
Tui.Form(...): TuiElement
Tui.Stack(Children; CurrentIndex: integer): TuiElement
Tui.Spacer.Fixed / Expanding
Tui.Label(Text: string): TuiElement
Tui.Button(Id: TuiControlId; Text: string; Action: TuiAction): TuiElement
Tui.Input(
  Id: TuiControlId;
  Text: string;
  Caret: integer;
  ChangeAction: TuiAction
): TuiElement
Tui.CheckBox(
  Id: TuiControlId;
  Text: string;
  Checked: boolean;
  ChangeAction: TuiAction
): TuiElement
Tui.List(
  Id: TuiControlId;
  Items: array of string;
  Selected: integer;
  ChangeAction: TuiAction
): TuiElement
Tui.Scroll(
  Id: TuiControlId;
  Offset: TuiPoint;
  ChangeAction: TuiAction;
  Child: TuiElement
): TuiElement
```

Layout modifiers (margins, spacing, size policy, stretch, alignment) attach as fields on
elements or as wrapper elements. Exact encoding is chosen for FPAS ergonomics during
implementation; the requirement is that they remain pure data.

### Messages and commands

```pascal
TuiControlId.Create(Value: integer): TuiControlId    { Value > 0 }
TuiAction.Create(Value: integer): TuiAction          { Value > 0 }

TuiMsg.Key(Key: TuiKeyEvent)
TuiMsg.Pointer(Pointer: TuiPointerEvent)
TuiMsg.Tick(DeltaMilliseconds: integer)
TuiMsg.Resize(Size: TuiSize)
TuiMsg.FocusChanged(
  Previous: option of TuiControlId;
  Current: option of TuiControlId
)
TuiMsg.Action(Source: TuiControlId; Action: TuiAction)
TuiMsg.TextChanged(
  Source: TuiControlId;
  Action: TuiAction;
  Text: string;
  Caret: integer
)
TuiMsg.CheckChanged(Source: TuiControlId; Action: TuiAction; Checked: boolean)
TuiMsg.SelectionChanged(Source: TuiControlId; Action: TuiAction; Selected: integer)
TuiMsg.ScrollChanged(Source: TuiControlId; Action: TuiAction; Offset: TuiPoint)
TuiMsg.QuitRequested

TuiCmd.None
TuiCmd.Quit
```

The dedicated controlled-value variants avoid optional payload bags and state exactly what the
runtime proposes as the next model value. Unhandled low-level input remains `Key` or `Pointer`.
There is no reserved action-id range inherited from Turbo Vision command constants.

`TuiCmd` deliberately starts with `None` and `Quit`. `Batch`, arbitrary closure-based `Post`, and
asynchronous effect results are outside v1 until a data-only application-message transport is
designed and proven in FPAS. Do not smuggle those effects through mutable widget or surface state.

## Explicitly absent

| Absent | Reason |
| --- | --- |
| `TuiView` live handles | Retained model abandoned. |
| `TuiContainer.Add` / `Remove` | Tree comes from `View`. |
| `TuiButton.OnClick` | Actions → messages. |
| `TuiDialog.OpenModal` | Modal-as-data in `View`. |
| `TuiCustomView.OnPaint` as object event | Optional custom element, not a live view. |
| Generational `Destroy` | Values are not registry objects. |

## Rough control set (phases)

Phase order follows [implementation-phases.md](implementation-phases.md):

1. Label, Button, Input
2. CheckBox, List, Scroll, Frame/Window/Dialog chrome
3. MenuBar, StatusLine
4. Memo / text viewer only if needed after the core feels right

Salvage painting and measurement ideas from Tui2 controls; do not salvage their handle APIs.
