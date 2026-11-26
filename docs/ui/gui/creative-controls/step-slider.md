# Step slider

The gallery selects one to four variations and offers a Generate action.
Available in **GUI**.

![Variation steps and Generate in the FPAS gallery](images/step-slider.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.StepSlider;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.ControlWells as Tools;
uses Fpas.Ui.Widgets.Inputs.StepSliders as W;
uses System as Sys;

// Chooses the number of variations to generate.
// ## Parameters
// - `Count`: Current number of variations from one through four.
// ## Returns
// The slider with stop changes and Generate requests.
public function View(Count: int): E.Element of W.Event;
begin
  return W.StepSlider of W.Event('variants', new W.Value('Variations', Count, 1, 4),
    function(Change: W.Event): W.Event => Change,
    new W.Options(Tool := new Tools.Tool(Sys.Char.ConvertFromUtf32($E145), 'Generate')));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Minimum / Maximum / Current` | Supply the first and last integer stops and a current stop in that range. |
| `Labels / Prefix / Suffix / Display` | All empty. Provide stop captions or format integer stops with a prefix and suffix. Display overrides the value caption. |
| `Tool / Plain / Enabled` | null / false / true. |

## Interaction

Drag or step through the stops with the keyboard. Store the proposed
value on a change. Use that stop for the operation when `Action` is true.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/StepSliders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [StepSlider.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/StepSlider.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.StepSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.StepSlider.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.StepSlider.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.StepSlider.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [step_slider.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/step_slider.c)
  - [step_slider.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/step_slider.h)
