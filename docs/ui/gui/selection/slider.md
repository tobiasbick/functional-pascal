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
    new C.Options(Step := 10.0F, ShowValue := true, Format := '%.0f%%'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Minimum / Maximum` | Finite bounds, with Maximum greater than Minimum. |
| `Value` | Pass a value within the bounds. |
| `Step` | 0 allows continuous adjustment. A positive value snaps changes to increments. |
| `ShowValue` | false. Set true to show the value beside the label. |
| `Format` | Null uses CUI's whole-number `%.0f` display. |
| `Enabled` | true. |
| `Width` | Fill. Content also uses native fill. Use Fixed or Percent to set a width. |

A format accepts literal text, `%%` for a percent sign, and one optional
floating-point conversion. Use `a`, `e`, `f`, or `g` in either case. Use literal
width and precision values of up to three digits. An empty format draws an
empty value.

## Interaction

Drag to adjust the value. With focus, use arrow keys and Shift for larger
adjustments. Use Home or End to move toward a bound. A positive step snaps
the proposal to increments measured from the minimum.

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
