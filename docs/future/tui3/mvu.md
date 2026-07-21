# Std.Tui3 Model–Update–View

Std.Tui3 uses The Elm Architecture (also called MVU). Inspiration: Elm, Bubble Tea, and
similar TEAs — not React hooks and not retained widget objects.

## Roles

| Piece | Contract |
| --- | --- |
| `Model` | Application-defined record (or small set of records). Single source of truth. |
| `TuiMsg` | Framework message. Includes raw fallback input and controlled-value changes. |
| `Update` | Returns the next `Model` and writes one closed `TuiCmd` output. |
| `View` | `Model → TuiElement`. Pure description of the next frame. |
| `TuiCmd` | Data-only runtime control (`NoCommand` or `Quit` in v1). |
| `TuiCmdOutput` | Host-owned capability through which `Update` returns one command. |

```text
TuiMsg ──► Update ──► Model
              │
              └──► TuiCmd ──► runtime control

Model ──► View ──► TuiElement ──► layout ──► paint
```

## Purity

- `Update` must not paint, read the terminal, or mutate element trees in place.
- `View` must be deterministic for a given `Model` and free of I/O.
- Layout and paint consume the element tree and must not call `Update`.
- `Update` may request only the closed runtime controls represented by `TuiCmd` v1.

Headless tests therefore exercise `Update` and `View` without a TTY.

## Messages

`TuiMsg` is a data-carrying enum owned by the library. The initial shape:

```text
TuiMsg =
  | Key of TuiKeyEvent
  | Pointer of TuiPointerEvent
  | Tick of integer          { delta milliseconds }
  | Resize of TuiSize
  | FocusChanged of Previous, Current control ids
  | Action of Source, Action
  | TextChanged of Source, Action, Text, Caret
  | CheckChanged of Source, Action, Checked
  | SelectionChanged of Source, Action, Selected
  | ScrollChanged of Source, Action, Offset
  | QuitRequested
```

Applications do not attach closures to buttons. A button stores a unique `TuiControlId` and a
repeatable `TuiAction`; activation becomes `TuiMsg.Action(Source, Action)`. Input, check, selection,
and scroll changes use distinct variants rather than an optional payload bag. Each contains the
complete proposed next controlled value.

Optional later: an application wrapper that maps `TuiMsg` into a private `AppMsg` enum.
That wrapper is sugar, not a required language feature.

## Control ids and actions

`TuiControlId` and `TuiAction` are distinct validated positive integers:

| Type | Contract |
| --- | --- |
| `TuiControlId` | Unique among interactive nodes in one rendered tree; identifies focus and message source. |
| `TuiAction` | Application intent; may be shared by several controls. |

Zero and negative values are invalid. Duplicate control ids are a diagnostic. Duplicate action ids
are intentional, for example two Save buttons that produce the same application intent. No numeric
range is reserved for old Turbo Vision command ids.

## Commands

`TuiCmd` v1 is deliberately closed and minimal:

| Command | Meaning |
| --- | --- |
| `NoCommand` | No effect (`None` is reserved in FPAS enum variants). |
| `Quit` | Request orderly shutdown. |

There is no closure-based `Post`: embedding a procedure would make the command executable behavior
rather than pure data and would not define how an application-specific result becomes a framework
message. `Batch`, timers, file dialogs, subscriptions, and worker results remain deferred until a
data-only message transport is designed and proven with FPAS's available type system. Commands
never mutate the `Model` directly.

## Run entry points

Required Phase 0 API shape:

```pascal
TuiApplication.OpenForTest(Size): TuiApplication
TuiApplication.RunIterations<TModel>(
  App: TuiApplication;
  InitialModel: TModel;
  Update: function(State: TModel; Msg: TuiMsg; Cmd: TuiCmdOutput): TModel;
  View: function(State: TModel): TuiElement;
  IterationCount: integer;
  DeltaMilliseconds: integer
): TModel

TuiApplication.Run<TModel>(
  InitialModel: TModel;
  Update: function(State: TModel; Msg: TuiMsg; Cmd: TuiCmdOutput): TModel;
  View: function(State: TModel): TuiElement
): TModel
```

`Run` acquires the interactive terminal, loops until quit, restores the terminal, and
returns the final model. `RunIterations` is the headless deterministic driver.

The runtime resets `TuiCmdOutput` to `NoCommand` before calling `Update`; `Update` calls
`Cmd.Set(TuiCmd.Quit)` when needed. A plain mutable enum parameter cannot be used as an output in
FPAS because parameter reassignment is local to the callee. The host-owned output capability
replaces the impossible `Model * TuiCmd` tuple sketch without requiring a generic result record.
**Proven compiling:**
[`tests/stdlib/tui3/mvu_host_signature_test.fpas`](../../../tests/stdlib/tui3/mvu_host_signature_test.fpas)
exercises `RunIterations<TModel>` with concrete `Update`/`View` callables. The element tree itself is
not generic over an application message type.

## Focus and control state

Focus owner (`option of TuiControlId`), caret position, input text, check state, list selection, and
scroll offsets are fields in the application `Model` (or nested records inside it). `View` passes
them into controlled element constructors; `Update` changes them in response to `TuiMsg`.

The runtime may help by:

- hit-testing the laid-out tree to decide which action or focus target a pointer hits;
- translating Tab / arrow navigation into `FocusChanged` when the tree marks focusable nodes;
- computing proposed controlled values from input without retaining them as hidden widget state;

but it does not keep a parallel mutable widget object graph as the source of truth.

## What this replaces from Tui2

| Tui2 | Tui3 |
| --- | --- |
| `Button.OnClick := ...` | `Tui.Button(ControlId, Text, ActionId)` + `TuiMsg.Action` |
| `Dialog.OpenModal()` | `if Model.Confirm then Tui.Dialog(...)` in `View` |
| `Desktop.Add(AsView(...))` | `Tui.Desktop(Focus, [ ... ])` returned from `View` |
| View lifecycle hooks | Frame rebuild; no attach/detach app API |
| Live action handles with `OnExecute` | Action ids and `Update` |

See [elements.md](elements.md) for the tree shape and [event-loop.md](event-loop.md) for
message production order.
