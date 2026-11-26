# Slider

Use a slider to adjust one value between two bounds. The gallery shows continuous volume and brightness in ten-point steps. Available in **GUI**.

![Slider in the FPAS gallery](images/slider.png)

## Example

```pascal
unit Docs.Ui.Selection.Slider;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.Sliders as C;

// Builds the slider example.
// ## Parameters
// - `Brightness`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Brightness: float): E.Element of float;
begin
  return C.Slider of float('brightness', 'Brightness', Brightness, 0.0F, 100.0F,
    function(Value: float): float => Value,
    new C.Options(Step := 10.0F, ShowValue := true, Percent := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Minimum / Maximum` | Finite bounds, with Maximum greater than Minimum. |
| `Value` | Pass a value within the bounds. |
| `Step` | 0 allows continuous adjustment. A positive value snaps changes to increments. |
| `ShowValue / DecimalPlaces / Percent` | false / 0 / false. Percent adds the percent suffix. |
| `Enabled` | true. |

## Interaction

Drag to adjust the value. With focus, use arrow keys, Shift for larger adjustments, or Home and End to reach the bounds. The callback receives the proposed value as a float.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_slider)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/Sliders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Selection/Widgets/Slider.fpas)
- [Range slider](range-slider.md)
- [Selection](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [Slider.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/Slider.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.Slider.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.Slider.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.Slider.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Slider.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [slider.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/slider.c)
  - [slider.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/slider.h)
