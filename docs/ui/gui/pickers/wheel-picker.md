# Wheel picker

Use a wheel to pick one caption at the center of a scrolling list. The gallery joins separate wheels for day, month and year. Available in **GUI**.

![Wheel picker in the FPAS gallery](images/wheel-picker.png)

## Example

```pascal
unit Docs.Ui.Pickers.WheelPicker;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Pickers.WheelPickers as W;

// Builds the wheel picker example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: int): E.Element of int;
begin
  return W.WheelPicker of int('picker-wheel-month', ['January', 'February', 'March', 'April', 'May',
    'June', 'July', 'August', 'September', 'October', 'November', 'December'],
    Selected, function(Index: int): int => Index,
    new W.Options(Width := L.Extent.Fixed(L.Measure.Logical(128)), Loop := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable wheel ID. |
| `Items / Selected` | Nonempty captions and a zero-based model index. CUI displays the part of a caption before `##`. |
| `Width` | `Content` uses 84 logical units. Also accepts `Fill`, a fixed logical width, or a percentage. |
| `Rows` | 5. Nonpositive uses 5. CUI rounds positive even counts up. |
| `Loop` | false. Continues from one end at the other when enabled. |
| `JoinLeft / JoinRight` | Both false. Each flag squares off that edge of the selection band. |
| `Enabled` | true. |

## Interaction

Drag the list or scroll over the wheel. Press Up or Down when the wheel has focus. The callback receives the proposed index. If several wheels form one date, resolve combinations such as February 31 in the application.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_wheel_picker)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/WheelPickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/WheelsAndDial.fpas)
- [Pickers](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [WheelPicker.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/WheelPicker.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.WheelPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.WheelPicker.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.WheelPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.WheelPicker.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [wheel_interruption.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/wheel_interruption.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [wheel_picker.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/wheel_picker.c)
  - [wheel_picker.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/wheel_picker.h)
