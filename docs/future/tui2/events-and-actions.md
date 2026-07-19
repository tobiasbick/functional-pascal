# Std.Tui2 events and actions

## Status and authority

Implemented application, custom-view lifecycle, raw-input, action, and semantic-button behavior is
documented in the current [`Std.Tui2` reference](../../pascal/std/tui2/README.md). This plan covers
the remaining typed-change and interactive control-routing surface.

## Design

Std.Tui2 uses three related mechanisms:

| Mechanism | Purpose |
| --- | --- |
| Direct event | One control- or lifecycle-specific callback such as `OnClick` or `OnPaint`. |
| `TuiAction` | One reusable user intention shared by buttons, menus, status items, and shortcuts. |
| Raw input event | An escape hatch for custom views and unhandled application input. |

All three use typed, single-handler event properties backed by the live registry. Std.Tui2 does not
provide a general publish/subscribe bus.

## Direct control events

Controls expose Pascal-style event properties backed by their live registry entries:

```pascal
Button.OnClick := HandleClick;

Input.OnChanged :=
  procedure(Sender: TuiInputLine; Value: string; Origin: TuiChangeOrigin)
  begin
    Preview.Text := Value
  end;
```

Initial control events are:

| Control | Event | Signature |
| --- | --- | --- |
| `TuiInputLine` | `OnChanged` | `procedure(Sender: TuiInputLine; Value: string; Origin: TuiChangeOrigin)` |
| `TuiCheckBox` | `OnChanged` | `procedure(Sender: TuiCheckBox; Value: boolean; Origin: TuiChangeOrigin)` |
| `TuiListBox` | `OnSelectionChanged` | `procedure(Sender: TuiListBox; Selected: integer; Origin: TuiChangeOrigin)` |
| `TuiRadioGroup` | `OnSelectionChanged` | `procedure(Sender: TuiRadioGroup; Selected: integer; Origin: TuiChangeOrigin)` |

Programmatic value changes follow one rule:

- user interaction raises the event with `TuiChangeOrigin.User`;
- normal property assignment raises it with `TuiChangeOrigin.Programmatic` only when the value changed;
- setting the current value raises nothing;
- the first API has no silent write operation.

The source application can be captured naturally when it is needed. It is not repeated in every
event signature merely to compensate for a missing closure feature.

## Actions

`TuiAction` remains a live application-owned handle representing one reusable operation. It owns:

- a positive `TuiCommand` identity;
- display text;
- an optional keyboard shortcut;
- enabled and visible state;
- optional checked state;
- one `OnExecute` event.

```pascal
var SaveAction: TuiAction := TuiAction.Create(App, CM_SAVE, 'Save');
var CtrlS: TuiKeyEvent := record
  kind := TuiKeyKind.Character;
  ch := 's';
  shift := false;
  ctrl := true;
  alt := false;
  meta := false;
end;

SaveAction.Shortcut := Some(CtrlS);
SaveAction.Enabled := CanSave;

SaveAction.OnExecute :=
  procedure(Sender: TuiAction; Source: TuiView)
  begin
    SaveDocument(Document)
  end;

SaveButton.Action := SaveAction;
SaveItem.Action := SaveAction;
```

Ordinary state uses computed record properties; events use the specialized event properties from
the language plan. Both resolve through accessors to the canonical live registry entry. Type-owned
functions remain for construction, conversions, commands, and operations that are not state-shaped.

Buttons, menu items, status items, shortcuts, tests, and direct command activation call
`TuiAction.Activate`. Activation verifies enabled state and raises `OnExecute` synchronously. The
implemented shortcut router runs after an unconsumed focused custom-view key handler and before the
application fallback. It matches every key field exactly; the first created enabled matching action
wins, and shortcut activation uses `TuiViewKind.Empty` as the source.

`TuiCommand` keeps a positive integer identity. Zero is invalid, `1..1023` is reserved for Std.Tui2,
and application commands start at `1024`.

## Direct events together with actions

A control may expose its semantic event and also be bound to an action. Their order is fixed:

1. internal control state is committed;
2. the bound action is activated when present and enabled;
3. the control's direct semantic event is raised;
4. invalidation is processed after dispatch returns.

The dispatcher revalidates the source handle after action execution. If the action destroyed or
detached the source, the later direct event is skipped. A panic stops the sequence and remains the
primary diagnostic.

This permits a reusable action plus one control-specific reaction without introducing multiple
subscribers. Applications should normally put the operation itself in the action and use the direct
event only for source-specific behavior.

## Custom-view lifecycle events

`TuiCustomView` exposes:

```text
OnAttach
OnDetach
OnMeasure
OnResize
OnPaint
OnFocus
OnBlur
OnCloseRequest
OnClosed
```

Their timing, purity, clipping, and re-entry rules remain those in `view-lifecycle.md`; only handler
registration changes from `Set...Handler` functions to event assignment.

`OnMeasure` and `OnCloseRequest` are function events. Tui2 checks `Assigned` before invocation:
default measurement uses the view's size hints, and an empty close request permits closing. Empty
raw boolean input events are treated as `false` by the router.

## Raw input events

Custom views assign raw handlers through their focused input capability:

```pascal
var Input: TuiViewInput := View.Input;
Input.OnKey :=
  function(Sender: TuiView; Key: TuiKeyEvent): boolean
  begin
    return false
  end;

Input.OnPointer := HandlePointer;
```

Application fallback events live on `App.Input` and receive input that view routing did not consume.
Returning `true` consumes the input; `false` leaves it unhandled. Raw events must not replace actions
for ordinary buttons, menus, or shortcuts.

## Application state

Capturing closures become the normal ownership mechanism for application state:

```pascal
procedure BuildCounter(App: TuiApplication; Parent: TuiContainer);
begin
  mutable var Count: integer := 0;
  var LabelView: TuiLabel := TuiLabel.Create(App, '0');
  var Button: TuiButton := TuiButton.Create(App, 'Increment');

  Button.OnClick :=
    procedure(Sender: TuiButton)
    begin
      Count := Count + 1;
      LabelView.Text := IntToStr(Count)
    end;

  TuiContainer.Add(Parent, TuiLabel.AsView(LabelView));
  TuiContainer.Add(Parent, TuiButton.AsView(Button))
end;
```

The registry entry behind the event property retains the closure environment, so `Count` and
`LabelView` remain valid after `BuildCounter` returns. Destroying the button clears its event
handlers and releases that environment.
Tui handles remain non-owning generational capabilities; capturing a handle does not keep its view
alive. A later invocation of a closure containing a stale handle receives the normal stale-handle
diagnostic.

Private unit state remains legal but is no longer the prescribed application architecture. Integer
tags remain optional association keys rather than a workaround for missing captures.

## Main-task and posting rules

Normal event assignment, clearing, and invocation are main-task-only because they mutate or use the
Tui2 live registry.

`TuiApplication.Post` accepts a procedure closure:

```pascal
TuiApplication.Post(
  App,
  procedure()
  begin
    Status.Text := 'Complete'
  end
);
```

Posting from the main task may retain task-bound mutable captures. Posting from a worker follows the
transfer rules in `docs/pascal/language/functions/closures.md`: captured input must be transferable and mutable captured
worker state is rejected. Posted callbacks run FIFO on the application main task and are discarded
if shutdown begins before they start.

## Event property ownership and destruction

The accessors behind an event property resolve the canonical optional handler owned by the live
registry entry. Copies and typed handle conversions resolve the same entry. Event assignment
therefore updates live registry state and adds no aliased event field to the handle record.

When a live object is destroyed:

1. remove it from routing and ownership structures;
2. clear every registry-backed event handler and release stored closures;
3. complete detach and close notifications according to lifecycle ordering;
4. invalidate the generational handle.

`OnClosed` is moved out before ordinary slots are cleared so it can run exactly once during orderly
destruction. Its closure is released immediately after invocation. Panic teardown clears remaining
slots without invoking user code.

## No general publish/subscribe bus

Each event contains zero or one handler. There are no subscription tokens, anonymous subscriber
lists, priorities, topic strings, bubbling semantic messages, or implicit cross-component delivery.
Input routing and focus traversal remain explicit Tui2 mechanisms; they are not public event-bus
subscriptions.

An application that needs domain-level fan-out may implement it explicitly using arrays of callable
values and define its own ordering and lifetime rules.

## Remaining implementation

1. Add typed value-change events and `TuiChangeOrigin` behavior.
2. Route action activation from menus and direct commands, then propagate it through view-backed
   controls.
3. Extend the headless post queue with worker-transfer enforcement.
4. Complete closure release, panic cleanup, and shutdown canaries.

## Remaining tests

- action activation from menu and direct command;
- change origin and no event for assignment of unchanged property values;
- captured stale handle diagnostic;
- event closure release on view destruction;
- callback panic preserves the primary diagnostic and restores terminal modes;
- posted closure FIFO and worker transfer rejection;
- headless tests do not require an interactive terminal.

## Remaining acceptance criteria

- typed-change events use the same registry-backed single-handler model;
- actions activate consistently from every supported source;
- posting preserves FIFO order and the task-transfer rules;
- destruction and panic paths release every remaining closure environment;
- complete Rust, FPAS, formatter, Clippy, and diff verification passes.

## Plan lifecycle

Delete this plan once its remaining behavior is implemented and described by current Tui2 docs and
regression tests.
