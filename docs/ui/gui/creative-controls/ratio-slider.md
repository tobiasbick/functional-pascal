# Ratio slider

Use a ratio slider for two shares that add up to one. The gallery shows
Coffee and Milk and a Train and Test split.
Available in **GUI**.

![Two ratio sliders in the FPAS gallery](images/ratio-slider.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.RatioSlider;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.RatioSliders as W;

// Sets the coffee fraction in a blend.
// ## Parameters
// - `Coffee`: Current coffee share from zero through one.
// ## Returns
// The slider with a proposed fraction as its message.
public function View(Coffee: float): E.Element of float;
begin
  return W.RatioSlider of float('blend', new W.Value('COFFEE', 'MILK', Coffee),
    function(Value: float): float => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `First / Second / Ratio` | Supply both captions and the first part's share from 0 through 1. The second share is one minus `Ratio`. |
| `Tone` | `Neutral` uses the theme accent for the first part. |
| `Step` | Zero uses one percentage point. A positive value sets the increment as a fraction of the range. |
| `Height` | Zero uses 52 logical units. |
| `Width` | `Fill`. Set a fixed or percentage width to constrain the slider. |
| `Enabled` | True. False disables pointer and keyboard input. |

## Interaction

Drag the divider or use the arrow keys. Shift multiplies the keyboard step by
ten. Home and End move the divider to the limits that keep the text readable.
The limits depend on the available width.

`Change` receives the proposed first share when input changes the value.
Store that share in the model. Keep the widget ID stable when either caption
changes to preserve focus.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_ratio_slider)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/RatioSliders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [RatioSlider.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/RatioSlider.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.RatioSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.RatioSlider.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.RatioSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.RatioSlider.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [ratio_slider.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/ratio_slider.c)
  - [ratio_slider.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/ratio_slider.h)
