# Std.Tui2 actions and handlers

Std.Tui2 separates low-level input routing from the semantic operations that application code handles.

## Three event layers

| Layer | Owner | Purpose |
| --- | --- | --- |
| Input events | TUI internals | Route key, mouse, focus, resize, and paste input. |
| Actions | Application code | Represent user intent such as Save, Close, Accept, or Open. |
| Typed change handlers | Application code | Observe a specific control value changing. |

Application code normally handles actions and typed changes. Raw key and mouse handlers are reserved for custom views and input that remained unhandled after normal routing.

## Actions

`TuiAction` is a live handle registered with one `TuiApplication`. It represents one reusable user operation.

An action may contain:

- a `TuiCommand` identity;
- display text;
- a keyboard shortcut;
- enabled state;
- visible state;
- optional checked state;
- exactly one action handler.

Bound controls reflect action text, enabled, visible, and checked state automatically. Updating an action invalidates every bound control affected by the change.

Buttons, menu items, status items, and shortcuts can refer to the same action. The operation and its availability therefore have one source of truth.

```text
button click ─────┐
menu selection ───┼─> TuiAction.Activate ─> action handler
keyboard shortcut ┘
```

The rough API is:

```pascal
TuiAction.New(
  App: TuiApplication;
  Command: TuiCommand;
  Text: string;
  Handler: TuiActionHandler
): TuiAction

TuiAction.Activate(Action: TuiAction; Source: TuiView)
TuiAction.SetEnabled(Action: TuiAction; Enabled: boolean)
TuiAction.SetVisible(Action: TuiAction; Visible: boolean)
TuiAction.SetChecked(Action: TuiAction; Checked: boolean)
TuiAction.SetShortcut(Action: TuiAction; Shortcut: TuiKey)
```

An application may activate an action by `TuiCommand`, allowing tests and application code to invoke the same path as interactive controls.

`TuiCommand` uses a positive integer identity. Zero is invalid, `1..1023` is reserved for Std.Tui2, and application commands start at `1024`. Negative identities are rejected.

## Button activation

A button does not publish a generic click message. It owns or references one `TuiAction`.

1. Internal routing delivers a mouse activation, Enter, Space, or matching mnemonic to the button.
2. The button updates its internal pressed state and consumes the input.
3. On a completed activation, it calls its action with the button's `TuiView` as `Source`.
4. The action verifies that it is enabled and invokes its handler synchronously.

The same action handler runs regardless of whether the action originated from a button, menu, shortcut, test, or direct command activation.

Example shape:

```pascal
procedure Save(App: TuiApplication; Action: TuiAction; Source: TuiView);
begin
  SaveDocument()
end;

var SaveAction: TuiAction :=
  TuiAction.New(App, CM_SAVE, 'Save', Save);

var SaveButton: TuiButton :=
  TuiButton.New(App, SaveAction);
```

## Typed change handlers

Controls with mutable values expose handlers named after the semantic change:

```pascal
TuiInputLine.SetChangedHandler(Input: TuiInputLine; Handler: TuiTextChangedHandler)
TuiCheckBox.SetChangedHandler(CheckBox: TuiCheckBox; Handler: TuiCheckedChangedHandler)
TuiListBox.SetSelectionHandler(List: TuiListBox; Handler: TuiListSelectionChangedHandler)
TuiRadioGroup.SetSelectionHandler(Group: TuiRadioGroup; Handler: TuiRadioSelectionChangedHandler)
```

Each control has at most one handler for each typed event. Setting a new handler replaces the previous handler. A separate clear operation removes it.

Typed change handlers receive the application, typed source handle, and new value. They are notifications and do not return `boolean`. Internal validation and state mutation complete before the notification runs.

Typed change handlers also receive `TuiChangeOrigin.User` or `TuiChangeOrigin.Programmatic`.

Programmatic setter behavior is fixed:

- user interaction emits the notification;
- a normal setter emits a programmatic notification only when the value actually changes;
- setting the current value emits nothing;
- the initial API has no silent setter.

## Raw input handlers

Custom views may register typed raw handlers for key and mouse input. The application may also have fallback handlers for input that no eligible view consumed.

Raw input handlers return `boolean`:

- `true` means the input was consumed;
- `false` continues normal routing or fallback handling.

Raw handlers must not replace actions for ordinary buttons, menus, or shortcuts.

## Callback execution

- Handlers execute synchronously on the application event-loop thread.
- Handler order is deterministic because each action or typed event has one handler.
- A handler may update views, actions, and application state.
- Mutations invalidate layout or paint as needed; redraw happens after dispatch returns.
- The dispatcher must revalidate live handles after a callback before continuing with affected objects.
- Nested application runs and uncontrolled callback re-entry are not part of the initial contract.

## No general publish/subscribe bus

The core does not provide multiple anonymous subscribers for arbitrary string or integer topics. This avoids subscription tokens, unspecified ordering, duplicate delivery, hidden dependencies, and callbacks that outlive their views.

An application may build a domain-specific message bus above Std.Tui2 when its architecture genuinely requires one.

## Non-blocking extensions

- Whether checked actions form an explicit mutually exclusive action group.
- Richer presentation overrides for one control bound to a shared action.

Initial handler signatures and application state ownership are defined in [application-state.md](application-state.md).
