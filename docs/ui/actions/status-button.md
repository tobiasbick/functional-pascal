# Status button

Show the progress and outcome of work in the button that started it. Available in **GUI and TUI**.

![Status button showing a successful save in the FPAS gallery](images/status-button.png)

## Example

```pascal
unit Docs.Ui.Actions.StatusButton;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Actions.StatusButtons as B;

// Builds the example view.
// ## Parameters
// - `State`: Current application value.
// ## Returns
// The widget with its message type.
public function View(State: B.Status): E.Element<string>;
begin
  return B.StatusButton<string>('save', 'Save changes', State, 'save',
    new B.Options(SuccessLabel := 'Saved'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `BusyLabel` | null uses the idle caption followed by an ellipsis |
| `SuccessLabel / ErrorLabel` | Done / Failed |
| `Variant / ControlSize / Enabled` | Primary / Medium / true |

## Interaction

The application owns Idle, Busy, Success and Error. Click, Enter or Space
sends the message only in Idle. Busy and result states remain focusable
while enabled. Use a background command to report completion and reset the
model when appropriate.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Actions/StatusButtons.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Actions/Widgets/StatusButton.fpas)
- [Actions](README.md)
