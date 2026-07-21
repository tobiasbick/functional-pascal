# `Std.Tui3`

`Std.Tui3` is the experimental source-level Model–Update–View terminal UI facade. Applications
return a fresh immutable `TuiElement` tree from `View`; they do not create, attach, or destroy live
widgets. The current implementation is headless and intended for deterministic tests.

## Quick reference

| Symbol | Purpose |
| --- | --- |
| `TuiSize.Create(Width, Height)` | Fixed headless terminal extent. |
| `TuiControlId.Create(Value)` | Positive focus and message-source identity. |
| `TuiAction.Create(Value)` | Positive application intent; values may repeat. |
| `TuiElement` | Closed data-carrying element enum. |
| `TuiElementBuilders` | Constructors for current elements. |
| `TuiMsg` | Key, tick, focus, action, text-change, and quit messages. |
| `TuiCmd` | `NoCommand` or `Quit`. |
| `TuiCmdOutput.Set(Command)` | Observable command output passed to `Update`. |
| `TuiApplication.OpenForTest(Size)` | Opens a fixed-size headless host. |
| `App.Inject(Msg)` | Queues one framework message. |
| `App.InjectKeyForTest(Key)` | Queues one key for focus/control routing. |
| `App.RunIterations(...)` | Processes a deterministic message budget. |
| `App.SurfaceSnapshot()` | Explicitly copies the last painted surface. |
| `App.Close()` | Closes the host and clears pending work. |

## Elements

`TuiElement` is a data-carrying enum. Its implemented variants are:

```pascal
TuiElement.Empty
TuiElement.Label(Text)
TuiElement.Button(Id, Text, Action)
TuiElement.Input(Id, Text, Caret, ChangeAction)
TuiElement.Row(Children)
TuiElement.Column(Children)
TuiElement.Window(Title, Children)
TuiElement.Dialog(Title, Children)
TuiElement.Desktop(Focused, Children)
```

`Row`, `Column`, `Window`, `Dialog`, and `Desktop` store recursive `array of TuiElement` payloads.
Interactive variants cannot omit their typed control or action identities. Validation before every
frame additionally rejects non-positive forged values, duplicate control ids, invalid input carets,
and focus ids that do not exist in the tree. Repeated action ids are valid.

## Update and View

The headless driver accepts this callable shape:

```pascal
function Update(
  State: AppModel;
  Msg: TuiMsg;
  Cmd: TuiCmdOutput
): AppModel;

function View(State: AppModel): TuiElement;
```

Set a command explicitly when needed:

```pascal
TuiMsg.QuitRequested:
begin
  Cmd.Set(TuiCmd.Quit);
  return State
end
```

`TuiCmdOutput` is a host-owned output capability because FPAS `mutable` value parameters permit
local reassignment only; they are not caller-visible output parameters. The host resets the output
to `NoCommand` before every `Update`, reads it immediately afterwards, and stops on `Quit` before
calling `View` or painting again.

## Headless frame and routing order

`RunIterations` renders an initial frame before consuming its iteration budget. Every processed
message then follows:

```text
dequeue → Update → command check → View → validate → layout → paint
```

Injected events are FIFO. Messages produced by one key remain ahead of the next external event.
Tab moves through the active focusable subtree. Character/editing keys produce controlled
`TextChanged` messages; Enter or Space activates a focused button. Escape produces
`QuitRequested`. When a dialog is present directly under the desktop, routing is limited to the
last such dialog subtree.

The current painter implements deterministic `Row`/`Column` preferred-size layout, full-size
windows, centered dialogs, borders, labels, controlled inputs, and buttons. The working surface is
host-owned. Painting replaces individual rows and does not construct a full-grid snapshot;
`SurfaceSnapshot` is the explicit copying boundary. Multi-column or multi-codepoint grapheme
clusters currently paint as `?`; full cell/style/grapheme storage remains subsequent Tui3
value-layer work.

## Implementation (contributors)

| Concern | Source |
| --- | --- |
| Elements and invariants | `lib/Std/Tui3/Elements/` |
| Geometry and measurement | `lib/Std/Tui3/Geometry/`, `lib/Std/Tui3/Layout/` |
| Working surface and paint | `lib/Std/Tui3/Rendering/` |
| Message loop and routing | `lib/Std/Tui3/Runtime/` |
| FPAS regressions | `tests/stdlib/tui3/` |

## See also

- [Standard library](../README.md)
- [Tui3 implementation plan](../../../future/tui3/README.md)
- [Testing](../testing/README.md)
