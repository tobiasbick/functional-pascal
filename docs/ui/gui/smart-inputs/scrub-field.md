# Scrub field

The gallery adjusts opacity, corner radius and blur with each value and its
unit in the same control. Available in **GUI**.

![Opacity, radius and blur scrub fields in the FPAS gallery](images/scrub-field.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.ScrubField;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.ScrubFields as W;

// Edits the corner radius.
// ## Parameters
// - `Radius`: Current radius between zero and 48 logical points.
// ## Returns
// A field that sends proposed radii to the update.
public function View(Radius: float): E.Element<float>;
requires Radius >= 0.0F and Radius <= 48.0F;
begin
  return W.ScrubField<float>('radius', 'Radius', Radius, 0.0F, 48.0F,
    function(Proposed: float): float => Proposed,
    new W.Options(Step := 0.5F, DecimalPlaces := 1, Suffix := 'px'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value / Minimum / Maximum` | Finite values with Minimum strictly less than Maximum. Value must be within the inclusive range. |
| `Step` | 0 selects one percent of the range. |
| `DecimalPlaces` | 0. Use 0 to 6 fractional digits for display. |
| `Suffix / Icon` | Empty. An icon replaces the caption. |
| `Width / Enabled` | Content / true. |

## Interaction

Drag sideways or use the arrow keys to adjust the value. Shift makes
dragging ten times finer and arrow steps ten times larger. Click to type
a value. CUI clamps a valid typed value to the bounds and retains the
model value on invalid input. Display precision leaves the underlying
number at its original precision.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/ScrubFields.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/Editors.fpas)
- [Number field](../inputs/number-field.md)
- [Smart inputs](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [ScrubField.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/ScrubField.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.ScrubField.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.ScrubField.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.ScrubField.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.ScrubField.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [scrub_field.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/scrub_field.c)
  - [scrub_field.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/scrub_field.h)
