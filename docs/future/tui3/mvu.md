# Std.Tui3 Model–Update–View

Std.Tui3 uses The Elm Architecture (also called MVU). Inspiration: Elm, Bubble Tea, and
similar TEAs — not React hooks and not retained widget objects.

## Roles

| Piece | Contract |
| --- | --- |
| `Model` | Application-defined record (or small set of records). Single source of truth. |
| `TuiMsg` | Framework message. Includes keys, pointer, tick, resize, and `Action(Id)`. |
| `Update` | `(Model, TuiMsg) → (Model, TuiCmd)`. Only place application state changes. |
| `View` | `Model → TuiElement`. Pure description of the next frame. |
| `TuiCmd` | Description of an effect. Runtime executes it; results return as later `TuiMsg`. |

```text
TuiMsg ──► Update ──► Model
              │
              └──► TuiCmd ──► effects ──► TuiMsg

Model ──► View ──► TuiElement ──► layout ──► paint
```

## Purity

- `Update` must not paint, read the terminal, or mutate element trees in place.
- `View` must be deterministic for a given `Model` and free of I/O.
- Layout and paint consume the element tree and must not call `Update`.
- Side effects leave `Update` only as `TuiCmd` values.

Headless tests therefore exercise `Update` and `View` without a TTY.

## Messages

`TuiMsg` is a data-carrying enum owned by the library. The initial shape:

```text
TuiMsg =
  | Key of TuiKeyEvent
  | Pointer of TuiPointerEvent
  | Tick of integer          { delta milliseconds }
  | Resize of TuiSize
  | Action of TuiAction      { element activation }
  | QuitRequested
```

Applications do not attach closures to buttons. A button stores a `TuiAction` id; activation
becomes `TuiMsg.Action(Id)`.

Optional later: an application wrapper that maps `TuiMsg` into a private `AppMsg` enum.
That wrapper is sugar, not a required language feature.

## Actions

`TuiAction` is a validated positive integer.

| Range | Owner |
| --- | --- |
| `1..1023` | Reserved for Std.Tui3 |
| `>= 1024` | Application |

Zero and negative ids are invalid. Duplicate ids in one tree are allowed; the runtime
documents which node wins (stable tree order, topmost interactive hit).

## Commands

`TuiCmd` starts small:

| Command | Meaning |
| --- | --- |
| `None` | No effect. |
| `Quit` | Request orderly shutdown. |
| `Batch of TuiCmd` | Run several commands. |
| `Post of procedure()` | Enqueue a main-task callback (worker bridge). |

Later commands (timers, file dialogs, custom subscriptions) return results as `TuiMsg`
values. Commands never mutate the `Model` directly.

## Run entry points

Conceptual API:

```pascal
TuiApplication.OpenForTest(Size): TuiApplication
TuiApplication.RunIterations(
  App;
  InitModel;
  Update;
  View;
  IterationCount;
  DeltaMilliseconds
): Model

TuiApplication.Run(
  InitModel;
  Update;
  View
): Model
```

`Run` acquires the interactive terminal, loops until quit, restores the terminal, and
returns the final model. `RunIterations` is the headless deterministic driver.

Exact signatures may use generic routines over `Model` where inference works. The element
tree itself is not generic over an application message type.

## Focus and control state

Focus owner, caret position, list selection, and scroll offsets are fields in the
application `Model` (or nested records inside it). `View` reads them; `Update` changes
them in response to `TuiMsg`.

The runtime may help by:

- hit-testing the laid-out tree to decide which action or focus target a pointer hits;
- translating Tab / arrow navigation into focus-move messages when the tree marks
  focusable nodes;

but it does not keep a parallel mutable widget object graph as the source of truth.

## What this replaces from Tui2

| Tui2 | Tui3 |
| --- | --- |
| `Button.OnClick := ...` | `Tui.Button(Text, ActionId)` + `TuiMsg.Action` |
| `Dialog.OpenModal()` | `if Model.Confirm then Tui.Dialog(...)` in `View` |
| `Desktop.Add(AsView(...))` | `Tui.Desktop([ ... ])` returned from `View` |
| View lifecycle hooks | Frame rebuild; no attach/detach app API |
| Live action handles with `OnExecute` | Action ids and `Update` |

See [elements.md](elements.md) for the tree shape and [event-loop.md](event-loop.md) for
message production order.
