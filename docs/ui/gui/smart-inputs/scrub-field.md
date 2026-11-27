# Scrub field

The gallery adjusts opacity, corner radius and blur with each value and its
unit in the same control. Available in **GUI**.

![Opacity, radius and blur scrub fields in the FPAS gallery](images/scrub-field.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.ScrubField;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Charts.Charts as C;
uses Fpas.Ui.Widgets.Inputs.ScrubFields as W;

// Edits the corner radius.
// ## Parameters
// - `Radius`: Current radius between zero and 48 logical points.
// ## Returns
// A field that sends proposed radii to the update.
public function View(Radius: float): E.Element of float;
requires Radius >= 0.0F and Radius <= 48.0F;
begin
  return W.ScrubField of float('radius', 'Radius', Radius, 0.0F, 48.0F,
    function(Proposed: float): float => Proposed,
    new W.Options(Step := 0.5F, Format := new C.ValueFormat(DecimalPlaces := 1), Suffix := 'px'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value / Minimum / Maximum` | Finite values with Minimum strictly less than Maximum. Value must be within the inclusive range. |
| `Step` | Zero selects one percent of the range. The fine step must support finite native rounding within the bounds. |
| [`Format`](../charts/README.md#value-format) | Null shows zero fractional digits. The format sets the resting value and the initial draft. |
| `Suffix` | Null omits the suffix. An empty string keeps its position and native spacing. |
| `Icon` | Null shows the caption. A glyph replaces the caption. An empty string hides it. |
| `Width` | Content. CUI prefers 160 logical units at rest and uses 160 units when editing. |
| `Enabled` | True. False disables editing and value changes. |

## Interaction

Drag sideways or use the arrow keys to adjust the value. Shift makes dragging
ten times finer and arrow steps ten times larger. Click without moving or press
Enter on the focused field to edit the number.

Press Enter or leave the field to commit a complete finite number. CUI clamps
the result to the bounds. Invalid text or Escape keeps the model value. Set a
unit with `Suffix` to keep the unit outside the numeric draft.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_scrub)
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
