# Std.Tui2 events and actions

## Status and authority

Planned Tui2 API contract. This document replaces the named-handler and unit-global-state decisions
currently described in `actions-and-handlers.md`, the fixed handler storage rules in
`application-state.md`, and the `Set...Handler` registration examples in `view-lifecycle.md`.
Those documents must be reconciled when this plan enters implementation.

The implementation depends on:

1. [capturing closures](../capturing-closures.md);
2. the bound-method milestone in [events and bound record methods](../events-and-bound-methods.md);
3. [record properties](../record-properties.md);
4. the event-property milestone in [events and bound record methods](../events-and-bound-methods.md).

It does not depend on the implementation details of expression postfix chaining. The chaining work
may finish independently and must not be modified as part of this plan.

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
| `TuiButton` | `OnClick` | `procedure(Sender: TuiButton)` |
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

SaveAction.Shortcut := TuiKey.Parse('Ctrl+S');
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

Buttons, menu items, status items, shortcuts, tests, and direct command activation all call
`TuiAction.Activate`. Activation verifies enabled state and raises `OnExecute` synchronously.

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

## Lifecycle events

Application lifecycle event members are:

```pascal
App.OnStart := procedure(Sender: TuiApplication) begin ... end;
App.OnStop := procedure(Sender: TuiApplication) begin ... end;
App.OnTick := procedure(Sender: TuiApplication; DeltaMilliseconds: integer) begin ... end;
```

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

Custom views may assign:

```pascal
View.OnKey :=
  function(Sender: TuiCustomView; Key: TuiKey): boolean
  begin
    return false
  end;

View.OnMouse := HandleMouse;
```

Application fallback events receive input that normal routing did not consume. Returning `true`
consumes the input; `false` continues to the next documented routing step. Raw events must not
replace actions for ordinary buttons, menus, or shortcuts.

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
transfer rules in `capturing-closures.md`: captured input must be transferable and mutable captured
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

## Superseded API shapes

The following planned shapes are removed before implementation:

```text
TuiAction.New(..., Handler)
TuiInputLine.SetChangedHandler(...)
TuiCheckBox.SetChangedHandler(...)
TuiListBox.SetSelectionHandler(...)
TuiRadioGroup.SetSelectionHandler(...)
TuiCustomView.SetPaintHandler(...)
TuiCustomView.SetResizeHandler(...)
TuiButton.SetText(...)
TuiButton.SetEnabled(...)
TuiButton.SetAction(...)
```

Construction no longer receives a mandatory handler. Events start empty or use an internal default
defined by the control. State-shaped getters and setters become properties; imperative operations
remain methods or type-owned functions.

## Implementation order

1. Complete capturing closures, bound methods, record properties, and event properties.
2. Add a small FPAS-only event canary outside Tui2.
3. Add registry-backed event properties to `TuiApplication`, `TuiAction`, and `TuiCustomView`.
4. Migrate application lifecycle and custom-view lifecycle registration.
5. Implement `TuiAction.OnExecute` and action bindings.
6. Implement button `OnClick` and the fixed action/direct-event ordering.
7. Add typed value-change events and origin behavior.
8. Add raw input and application fallback events.
9. Add closure release, stale-handle, panic, and shutdown canaries.
10. Reconcile every Tui2 planning document and continue the existing phase sequence.

## Required tests

- named routine, bound method, and capturing closure assigned to Tui2 events;
- handler replacement and clearing through event accessors;
- copied and converted handles resolving the same registry-backed event property;
- action activation from button, menu, shortcut, test, and command;
- action-before-direct-event ordering;
- source destruction during action execution skips the direct event safely;
- change origin and no event for assignment of unchanged property values;
- raw input consumed and unconsumed paths;
- closure-captured local surviving builder return;
- captured stale handle diagnostic;
- event closure release on view destruction and application shutdown;
- callback panic preserves the primary diagnostic and restores terminal modes;
- posted closure FIFO and worker transfer rejection;
- headless tests do not require an interactive terminal.

## Acceptance criteria

- Tui2 application code can use Pascal-style `On... := Handler` syntax;
- ordinary events and actions have separate, documented purposes;
- each event has deterministic single-handler behavior;
- local application state no longer requires unit globals or integer tags;
- registry-backed properties prevent handlers from being lost through handle copies;
- destruction and panic paths release every closure environment;
- ordinary and event properties remain distinct from a publish/subscribe bus;
- old `Set...Handler` plan text is removed when implementation starts;
- current `docs/pascal/` documentation is changed only after the API exists;
- complete Rust, FPAS, formatter, Clippy, and diff verification passes.

## Plan lifecycle

Keep this document as the authoritative future event contract until Tui2 implements it. During
implementation, mark completed steps here and make the next step explicit. Once current Tui2 docs
and regression tests describe the shipped API, delete this plan and remove its future references.
