# Ruler slider

Use a ruler slider to move a measurement tape under a fixed marker. The gallery shows height in centimeters and temperature in degrees Celsius. Available in **GUI**.

![Ruler slider in the FPAS gallery](images/ruler-slider.png)

## Example

```pascal
unit Docs.Ui.Pickers.RulerSlider;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.RulerSliders as W;

// Builds the ruler slider example.
// ## Parameters
// - `Temperature`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Temperature: float): E.Element<float>;
begin
  return W.RulerSlider<float>('picker-temperature', 'Temperature', Temperature, 10.0F, 30.0F,
    function(Value: float): float => Value,
    new W.Options(Step := 0.5F, MajorEvery := 4, Spacing := 14.0F, DecimalPlaces := 1, UnitLabel := '°C'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value / Minimum / Maximum` | Finite value within increasing, inclusive bounds. |
| `Step / MajorEvery` | 1 / 10. The step is the value per tick and MajorEvery counts ticks between labels. |
| `Spacing / Width` | 9 / 0 logical points. Zero width fills available space. |
| `DecimalPlaces / UnitLabel` | 0 / empty. Precision accepts 0 through 6. |
| `Enabled` | true. |

## Interaction

Drag the tape, scroll over it or use the arrow keys while it has focus. Shift with an arrow moves one major tick interval. Apply the proposed measurement in the model.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/RulerSliders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/RulerAndHistogram.fpas)
- [Pickers](README.md)
