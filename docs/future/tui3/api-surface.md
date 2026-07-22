# Std.Tui3 API surface

This is the target contract for the implementation tasks. Symbols remain planned until they are
implemented and documented under `docs/pascal/std/tui3/`. A task may narrow the contract only by
updating this file before code; it must not choose a different encoding silently.

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
| `TuiCmdOutput` | Host-owned observable output capability passed to `Update`. |
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
  Update: function(State: TModel; Msg: TuiMsg; Cmd: TuiCmdOutput): TModel;
  View: function(State: TModel): TuiElement;
  IterationCount: integer;
  DeltaMilliseconds: integer
): TModel
TuiApplication.SurfaceSnapshot(App): TuiSurfaceSnapshot
TuiApplication.Close(App)

TuiApplication.Run<TModel>(
  InitialModel: TModel;
  Update: function(State: TModel; Msg: TuiMsg; Cmd: TuiCmdOutput): TModel;
  View: function(State: TModel): TuiElement
): TModel
```

The runtime resets a host-owned `TuiCmdOutput` to `NoCommand` before every `Update` call. `Update`
uses `Cmd.Set(...)`, and the runtime reads the output before another `View`. A plain `mutable
TuiCmd` parameter is not an output parameter in FPAS: reassignment changes only the callee's local
binding. The capability avoids a generic result record while keeping `TuiCmd` itself data-only.

### Element constructors

```pascal
TuiElementBuilders.MakeEmpty(): TuiElement
TuiElementBuilders.MakeDesktop(Focused: option of TuiControlId; Children: array of TuiElement): TuiElement
TuiElementBuilders.MakeWindow(Title: string; Children: array of TuiElement): TuiElement
TuiElementBuilders.MakeDialog(Title: string; Children: array of TuiElement): TuiElement
TuiElementBuilders.MakeMenuBar(Items: array of TuiMenuItem): TuiElement
TuiElementBuilders.MakeStatusLine(Items: array of TuiStatusItem): TuiElement
TuiElementBuilders.MakeRow(Children: array of TuiElement): TuiElement
TuiElementBuilders.MakeRowSpaced(Children: array of TuiElement; Spacing: integer): TuiElement
TuiElementBuilders.MakeColumn(Children: array of TuiElement): TuiElement
TuiElementBuilders.MakeColumnSpaced(Children: array of TuiElement; Spacing: integer): TuiElement
TuiElementBuilders.MakeLayout(Settings: TuiLayoutSettings; Child: TuiElement): TuiElement
TuiElementBuilders.MakeSpacer(TuiSpacer.Fixed / Expanding): TuiElement
TuiElementBuilders.MakeLabel(Text: string): TuiElement
TuiElementBuilders.MakeButton(Id: TuiControlId; Text: string; Action: TuiAction): TuiElement
TuiElementBuilders.MakeInput(
  Id: TuiControlId;
  Text: string;
  Caret: integer;
  ChangeAction: TuiAction
): TuiElement
TuiElementBuilders.MakeCheckBox(
  Id: TuiControlId;
  Text: string;
  Checked: boolean;
  ChangeAction: TuiAction
): TuiElement
TuiElementBuilders.MakeList(
  Id: TuiControlId;
  Items: array of string;
  Selected: integer;
  ChangeAction: TuiAction
): TuiElement
TuiElementBuilders.MakeScroll(
  Id: TuiControlId;
  Offset: TuiPoint;
  ChangeAction: TuiAction;
  Child: TuiElement
): TuiElement
```

#### List selection (Gate 4.A — frozen)

`Selected` is model data. Validation requires:

- empty `Items` → `Selected = -1`;
- non-empty `Items` → `Selected` in `0 .. Length(Items) - 1`.

Routing proposes only a valid next index. Focused keyboard behavior:

| Key | Proposal |
| --- | --- |
| Up | `Selected - 1`, clamped to `0` |
| Down | `Selected + 1`, clamped to `Length - 1` |
| Home | `0` |
| End | `Length - 1` |

Empty lists leave the key as `TuiMsg.Key`. When the proposed index equals the current
`Selected`, routing does not emit `SelectionChanged`.

Left-button pointer down maps `Y - Bounds.Y` to a row index inside `0 .. Length - 1`. Hits
outside that range remain `TuiMsg.Pointer`. Focus changes precede `SelectionChanged` when needed.

Viewport rule: List has no scroll offset field. Preferred height is `max(1, Length(Items))`;
preferred width is `2 + max item display width` (minimum width `2`). Paint always starts at item
`0` and draws at most `Bounds.Height` rows; clipped rows are not painted. Wrap the list in
`Scroll` when the model needs a scrolled viewport.

`TuiElementBuilders.MakeRow` and `MakeColumn` preserve their Phase 0 signatures and use zero
spacing. `MakeRowSpaced` and `MakeColumnSpaced` validate and store non-negative spacing.
`TuiElement.Layout` is encoded internally as a recursive enum variant whose children array must contain
exactly one element; its public builder accepts one `Child`. This keeps recursion behind the same
array indirection already proven by Phase 0. Validation rejects forged zero-child or multi-child
layout wrappers.

#### Scroll clamping (Gate 4.B — frozen)

`Scroll(Id, Offset, ChangeAction, Child)` is a controlled single-child viewport. Its internal
children array contains exactly `Child`; validation requires a positive id and action, exactly one
child, and non-negative `Offset.X` and `Offset.Y`.

Content size is the child preferred size measured with an unbounded spec. The viewport is the
arranged Scroll bounds. For each axis, `MaxOffset = max(0, Content - Viewport)` and the effective
offset used to arrange, paint, and hit-test is the model offset clamped to `0 .. MaxOffset`. The
runtime does not retain a clamped offset or scrollbar state: it only proposes
`ScrollChanged(Source, Action, Offset)` for the application model to accept.

When Scroll has focus, Left/Right/Up/Down propose a one-cell delta, PageUp/PageDown propose a
vertical delta of the viewport height, Home proposes `Y = 0`, and End proposes `Y = MaxOffset.Y`.
Proposals are clamped; when a proposal equals the current model offset, the input remains
`TuiMsg.Key`. Left pointer down inside the viewport focuses Scroll when necessary; no scrollbar
widgets are invented. Child hits are considered only inside the Scroll clip and use offset-aware
child geometry.

`Std.Console.MouseAction` currently exposes `ScrollUp`/`ScrollDown`, not `WheelUp`/`WheelDown`;
therefore this version does not route wheel events as Scroll changes. If the console contract adds
the named wheel variants, WheelUp/WheelDown route over the viewport without a left button and
propose a clamped vertical delta of `-3`/`+3`.

The v1 layout values are the Tui-prefixed value semantics already proven by Tui2:

```pascal
TuiSizePolicyKind = (Fixed, Minimum, Maximum, Preferred, Expanding)
TuiSizePolicy(Horizontal, Vertical: TuiSizePolicyKind)

TuiAlignmentKind = (Leading, Center, Trailing, Fill)
TuiAlignment(Horizontal, Vertical: TuiAlignmentKind)

TuiMargins(Left, Top, Right, Bottom: integer)       { all non-negative }
TuiLayoutSettings(
  Margins: TuiMargins;
  SizePolicy: TuiSizePolicy;
  Alignment: TuiAlignment;
  Stretch: integer                                  { non-negative }
)

TuiMeasureConstraintKind = (Unbounded, AtMost)
TuiMeasureConstraint(Kind; Limit: integer)          { bounded limit non-negative }
TuiMeasureSpec(Width, Height: TuiMeasureConstraint)
TuiMeasureResult(Minimum, Preferred, Maximum: TuiSize)
TuiSpacerKind = (Fixed, Expanding)
TuiSpacer(Kind; MinimumExtent: integer)              { non-negative }
TuiLayoutFit(Minimum, Available, Overflow: TuiSize)
```

Defaults are zero margins, preferred size policy, fill alignment, and stretch zero. The static
constructors and invariants match the corresponding files under `lib/Std/Tui2/Layouts/`, renamed
to Tui3. `TuiMeasureResult` requires `Minimum <= Preferred <= Maximum` on both axes.

Grid, Form, Stack, stable list keys, and custom paint are not part of Phases 1–5. Add a separate
planned task and exact declarations before implementing any of them.

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

TuiCmd.NoCommand
TuiCmd.Quit
```

`None` is a reserved token in FPAS enum variants, so the idle command is `NoCommand` rather than
`None`. The empty element constructor is `TuiElement.Empty` for the same reason.

The dedicated controlled-value variants avoid optional payload bags and state exactly what the
runtime proposes as the next model value. Unhandled low-level input remains `Key` or `Pointer`.
There is no reserved action-id range inherited from Turbo Vision command constants.

`TuiCmd` deliberately starts with `NoCommand` and `Quit`. `Batch`, arbitrary closure-based `Post`, and
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

## Chrome data (Gates 4.C / 4.D — frozen)

v1 MenuBar is a **flat action bar** (no open/dropdown menus, no Turbo Vision command offsets).
StatusLine replaces the earlier `StatusLine(Text: string)` sketch with an item array.

```pascal
TuiMenuItem = record
  Id: TuiControlId;
  Text: string;
  Action: TuiAction;
  Enabled: boolean;
  Shortcut: string;   { display hint only in v1; not auto-bound to keys }
end;

TuiMenuItem.Create(Id, Text, Action): TuiMenuItem           { Enabled := true; Shortcut := '' }
TuiMenuItem.Disabled(Id, Text, Action): TuiMenuItem        { Enabled := false }
TuiMenuItem.WithShortcut(Id, Text, Action, Shortcut): TuiMenuItem

TuiElementBuilders.MakeMenuBar(Items: array of TuiMenuItem): TuiElement

TuiStatusItem = enum
  Hint(Text: string);                                      { display-only; not focusable }
  Command(Id: TuiControlId; Text: string; Action: TuiAction; Enabled: boolean);
end;

TuiStatusItemBuilders.MakeHint(Text): TuiStatusItem
TuiStatusItemBuilders.MakeCommand(Id, Text, Action): TuiStatusItem { Enabled := true }
TuiStatusItemBuilders.MakeDisabledCommand(Id, Text, Action): TuiStatusItem

TuiElementBuilders.MakeStatusLine(Items: array of TuiStatusItem): TuiElement
```

### MenuBar interaction

- Preferred height `1`. Preferred width is the sum of item label widths plus separators
  (` Text ` around each item; disabled items still occupy space).
- Enabled items are focusable in left-to-right order; disabled items are skipped by Tab and
  reject activation.
- Enter/Space on a focused enabled item emits `TuiMsg.Action(Source, Action)`.
- Left-button pointer down on an enabled item focuses it (if needed) then emits `Action`.
- Hits on disabled items or empty bar cells remain `TuiMsg.Pointer`.
- No model-controlled open menu state in v1.

### StatusLine interaction

- Preferred height `1`. Preferred width is the joined item text widths with single-space gaps.
- `Hint` items are never focusable.
- Enabled `Command` items are focusable; disabled commands are not.
- Enter/Space or left-click on an enabled command emits `TuiMsg.Action`.
- A status line that contains only `Hint` items has no focusable controls.
