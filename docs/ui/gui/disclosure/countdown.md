# Countdown

Use a countdown to show remaining time. The gallery displays standard tiles and a compact hours, minutes and seconds layout. Available in **GUI**.

![Countdown in the FPAS gallery](images/countdown.png)

## Example

```pascal
unit Docs.Ui.Disclosure.Countdown;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Feedback.Countdowns as W;

// Builds the countdown example.
// ## Parameters
// - `SecondsLeft`: Time remaining from the application clock.
// ## Returns
// The widget for the application view.
public function View(SecondsLeft: double): E.Element<string>;
begin
  return W.Countdown<string>('disclosure-sale', SecondsLeft, new W.Options(Compact := true, HideDays := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `SecondsLeft` | The remaining duration in seconds from the application clock. |
| `Compact / HideDays` | false / false. HideDays folds days into hours. |

## Interaction

Refresh the remaining time from a clock in your application. The control displays that value and CUI rolls the digits as it changes.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Countdowns.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Disclosure/Widgets/Countdown.fpas)
- [Disclosure](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [Countdown.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Countdown.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Countdown.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Countdown.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Countdown.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Countdown.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [countdown.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/countdown.c)
  - [countdown.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/countdown.h)
