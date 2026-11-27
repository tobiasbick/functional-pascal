# Range slider

Use a range slider for two ordered endpoints, such as a minimum and maximum price. Available in **GUI**.

![Range slider in the FPAS gallery](images/range-slider.png)

## Example

```pascal
unit Docs.Ui.Selection.RangeSlider;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.RangeSliders as C;

// Builds the range slider example.
// ## Parameters
// - `Price`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Price: C.Interval): E.Element of C.Interval;
begin
  return C.RangeSlider of C.Interval('price-range', 'Price range', Price, 0.0F, 100.0F,
    function(Value: C.Interval): C.Interval => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value` | An Interval with Lower less than or equal to Upper. Keep both endpoints within the bounds. |
| `Minimum / Maximum` | Finite bounds, with Maximum greater than Minimum. |
| `Enabled` | true. |

CUI uses the available width and displays both endpoints as whole numbers.
Use `Elements.Size` to set the width through layout.

## Interaction

Press the track to move the nearest endpoint and drag to adjust it. CUI keeps the endpoints ordered. With the track focused, Space switches the active endpoint. Arrow keys, Shift and Home or End adjust it. The callback receives both endpoints in one message.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_range_slider)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/RangeSliders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Selection/Widgets/RangeSlider.fpas)
- [Slider](slider.md)
- [Selection](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [RangeSlider.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/RangeSlider.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.RangeSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.RangeSlider.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.RangeSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.RangeSlider.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [range_slider.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/range_slider.c)
  - [range_slider.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/range_slider.h)
