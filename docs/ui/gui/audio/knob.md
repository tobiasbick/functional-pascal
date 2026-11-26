# Knob

Use a knob to adjust a value in little space. The gallery shows logarithmic
Cutoff and Attack controls, linear Resonance and a bipolar Pan control.
Available in **GUI**.

![Synth knobs in the FPAS gallery](images/knob.png)

## Example

```pascal
unit Docs.Ui.Audio.Knob;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.Knobs as W;

// Adjusts the cutoff of a filter.
// ## Parameters
// - `Hertz`: Current cutoff between 20 and 20,000 Hz.
// ## Returns
// The knob, with each proposed cutoff as an application message.
public function View(Hertz: float): E.Element of float;
begin
  return W.Knob of float('cutoff', 'Cutoff', Hertz, 20.0F, 20000.0F,
    function(Value: float): float => Value,
    new W.Options(UnitLabel := 'Hz', LogScale := true, Reset := 1000.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value / Minimum / Maximum` | Supply finite values with increasing bounds. |
| `Step / Reset` | 0 / 0. Zero Step allows continuous input. CUI clamps Reset to the bounds. |
| `LogScale / Bipolar` | Both false. LogScale requires a positive minimum. Bipolar draws the arc from the middle of the sweep. |
| `DecimalPlaces / UnitLabel / ShowSign / HideValue` | 0 / empty / false / false. |
| `Size / ArcColor / Enabled` | 48 logical points / null / true. |

## Interaction

Drag vertically or focus the knob and use the wheel or arrows. Shift makes
dragging finer and increases the arrow-key step. Double-click, Delete or
Backspace restores Reset. Store the proposed value to accept the change.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_knob)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/Knobs.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)
- [Audio](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [Knob.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/Knob.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.Knob.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.Knob.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.Knob.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.Knob.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [knob.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/knob.c)
  - [knob.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/knob.h)
