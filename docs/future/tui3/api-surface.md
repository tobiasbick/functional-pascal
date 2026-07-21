# Std.Tui3 API surface

This inventory is a planning sketch, not a frozen specification. Implemented symbols will
be documented under `docs/pascal/std/tui3/` only after they exist.

## Type categories

### Value records

| Type | Purpose |
| --- | --- |
| `TuiPoint` / `TuiSize` / `TuiRect` | Geometry. |
| `TuiColor` / `TuiStyle` / `TuiStyleRole` / `TuiCell` / `TuiPalette` | Painting values. |
| `TuiSurface` | Retained cell grid for headless inspection and terminal flush. |
| `TuiCanvas` | Clipped drawing value for one paint pass. |
| `TuiSizePolicy` / `TuiMargins` / `TuiAlignment` / `TuiSpacer` | Layout inputs. |
| `TuiMeasureSpec` / `TuiMeasureResult` / `TuiLayoutFit` | Measurement results. |
| `TuiKeyEvent` | Alias of `Std.Console.KeyEvent`. |
| `TuiPointerEvent` | Normalized pointer action, button, position, modifiers. |
| `TuiAction` | Positive action identity. |
| `TuiMsg` | Framework message delivered to `Update`. |
| `TuiCmd` | Effect request from `Update`. |
| `TuiElement` | Immutable UI node description. |
| `TuiMenuItem` / `TuiStatusItem` | Chrome descriptions. |

These records do not own live widgets.

### Application host

| Type | Purpose |
| --- | --- |
| `TuiApplication` | Owns the run loop, headless surface, and terminal session when interactive. |

`TuiApplication` is a host, not a view registry for application widgets.

## Core operations

### MVU

```pascal
TuiApplication.OpenForTest(Size: TuiSize): TuiApplication
TuiApplication.Inject(App; Msg: TuiMsg)
TuiApplication.RunIterations(
  App: TuiApplication;
  Model: Model;
  Update: function(Model; TuiMsg): Model * TuiCmd;
  View: function(Model): TuiElement;
  IterationCount: integer;
  DeltaMilliseconds: integer
): Model
TuiApplication.Surface(App): TuiSurface
TuiApplication.Close(App)

TuiApplication.Run(
  Model: Model;
  Update: function(Model; TuiMsg): Model * TuiCmd;
  View: function(Model): TuiElement
): Model
```

Generic routine parameters over `Model` are preferred when inference is reliable.

### Element constructors (illustrative)

```pascal
Tui.None: TuiElement
Tui.Desktop(Children: array of TuiElement): TuiElement
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
Tui.Button(Text: string; Action: TuiAction): TuiElement
Tui.Input(Text: string; Action: TuiAction): TuiElement
Tui.CheckBox(Text: string; Checked: boolean; Action: TuiAction): TuiElement
Tui.List(Items: array of string; Selected: integer; Action: TuiAction): TuiElement
Tui.Scroll(Offset: TuiPoint; Child: TuiElement): TuiElement
```

Layout modifiers (margins, spacing, size policy, stretch, alignment) attach as fields on
elements or as wrapper elements. Exact encoding is chosen for FPAS ergonomics during
implementation; the requirement is that they remain pure data.

### Messages and commands

```pascal
TuiAction.Create(Value: integer): TuiAction          { Value >= 1024 }
TuiAction.FromStandard(Value: integer): TuiAction    { internal 1..1023 }

TuiMsg.Key / Pointer / Tick / Resize / Action / QuitRequested
TuiCmd.None / Quit / Batch / Post
```

Input-line and list activations need payloads. Preferred shape:

```text
TuiMsg.Action of record
  Id: TuiAction;
  Text: option of string;     { input commit / edit }
  Index: option of integer;   { list selection }
end
```

or distinct variants `TextEdited`, `Selected` that still carry the action id. Avoid
callbacks.

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
