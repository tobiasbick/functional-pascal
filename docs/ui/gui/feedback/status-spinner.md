# Status spinner

Use a status spinner for work that moves from an activity indicator to a success or error marker. Available in **GUI**.

![Status spinner in the FPAS gallery](images/status-spinner.png)

## Example

```pascal
unit Docs.Ui.Feedback.StatusSpinner;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.StatusButtons as Work;
uses Fpas.Ui.Widgets.Feedback.StatusSpinners as W;

// Builds the status spinner example.
// ## Parameters
// - `Status`: Current state of the upload.
// ## Returns
// The widget for the application view.
public function View(Status: Work.Status): E.Element of string;
begin
  return W.StatusSpinner of string('feedback-upload-status', Status, new W.Options(Size := 18.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Status` | Pass Idle, Busy, Success or Error from the model. |
| `Size / Color` | 0 uses the native diameter of 20 logical points. Automatic uses the theme's busy ring color. |
| `Keep` | false. Set true to retain the result marker. |

## Interaction

Set Busy when work starts, then pass Success or Error when it finishes. Idle has no indicator. CUI animates the ring and expires the result marker unless Keep is enabled. The gallery pairs it with an Upload button and a caption.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_status_spinner)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/StatusSpinners.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/Loading.fpas)
- [Spinner](spinner.md)
- [Status button](../actions/status-button.md)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [StatusSpinner.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/StatusSpinner.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.StatusSpinner.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.StatusSpinner.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.StatusSpinner.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.StatusSpinner.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [status_spinner.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/status_spinner.c)
  - [status_spinner.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/status_spinner.h)
