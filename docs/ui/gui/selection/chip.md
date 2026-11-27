# Chip

Use chips for independently selected categories. The gallery places them in a row that wraps when space runs out. Available in **GUI**.

![Chip in the FPAS gallery](images/chip.png)

## Example

```pascal
unit Docs.Ui.Selection.Chip;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.Chips as C;

// Builds the chip example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: bool): E.Element of bool;
begin
  return C.Chip of bool('category-design', 'Design', Selected,
    function(Value: bool?): bool => Value ?? false);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Selected` | Pass the current Boolean from the model or null for a stateless action. |
| `Icon` | Null omits the icon. An empty string keeps its space while the chip is unselected. |
| `Enabled` | true. |

## Interaction

Click the chip or press Enter or Space while it has focus. A selectable chip
proposes the opposite Boolean. CUI draws a check mark for the selected state.
A stateless chip passes null to the message factory and keeps its unselected
appearance.

Keep a separate value for each independent selection. The stable ID
preserves focus when the literal caption changes.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_chip)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/Chips.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Selection/Widgets/Chip.fpas)
- [Selection](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [Chip.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/Chip.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.Chip.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.Chip.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.Chip.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Chip.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [chip.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/chip.c)
  - [chip.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/chip.h)
- [CuiChipRatingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiChipRatingTests.cs)
