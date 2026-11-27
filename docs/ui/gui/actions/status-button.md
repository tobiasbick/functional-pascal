# Status button

Show the progress and outcome of work in the button that started it. Available in **GUI**.

![Status button showing a successful save in the FPAS gallery](images/status-button.png)

## Example

```pascal
unit Docs.Ui.Actions.StatusButton;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.StatusButtons as B;

// Builds the example view.
// ## Parameters
// - `State`: Current application value.
// ## Returns
// The widget with its message type.
public function View(State: B.Status): E.Element of string;
begin
  return B.StatusButton of string('save', 'Save changes', State, 'save',
    new B.Options(SuccessLabel := 'Saved'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `BusyLabel` | Null uses the idle caption followed by an ellipsis. |
| `SuccessLabel` | `Done`. |
| `ErrorLabel` | `Failed`. |
| `Icon` | Null omits the idle glyph. An empty string reserves the glyph's space. |
| `Variant` | `Primary`. `Default` also uses the native primary appearance. Other choices are `Secondary`, `Destructive`, `Outline`, `Ghost` and `Link`. |
| `ControlSize` | `Medium`. Also accepts `Small` and `Large`. |
| `Enabled` | True. False rejects activation and focus. |

## Interaction

The application owns Idle, Busy, Success and Error. Click, Enter or Space
sends the message only in Idle. Busy and result states remain focusable
while enabled. CUI draws a spinner, check or error glyph. Use an empty caption
to show the corresponding glyph on its own.

Use a background command to report completion and reset the model. The gallery
shows Busy for 1.4 seconds and Saved for 1.6 seconds. The host wakes when it
receives the timed messages. [Redraw requests](../host.md#invalidation-and-inspection)
repaint a view without advancing its model.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_status_button)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/StatusButtons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/StatusButton.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui/Internal/Actions/`
  - [StatusButtons.fpas](../../../../src/Fpas.Ui/Internal/Actions/StatusButtons.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [StatusButton.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/StatusButton.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.StatusButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.StatusButton.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.StatusButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.StatusButton.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [status_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/status_button.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [status_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/status_button.c)
  - [status_button.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/status_button.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Actions/`
  - [CuiStatusButtonTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Actions/CuiStatusButtonTests.cs)
