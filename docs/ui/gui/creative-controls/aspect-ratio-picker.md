# Aspect ratio picker

Use an aspect ratio picker to choose the shape of a picture or frame.
The gallery shows six presets and a Custom choice.
Available in **GUI**.

![Aspect-ratio shapes in the FPAS gallery](images/aspect-ratio-picker.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.AspectRatioPicker;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Pickers.AspectPickers as W;

// Shows the output ratio picker.
// ## Parameters
// - `Selected`: Zero-based ratio index, including the final Custom slot.
// ## Returns
// The picker with the proposed index as its message.
public function View(Selected: int): E.Element of int;
begin
  return W.AspectPicker of int('output-ratio', new W.Value('Aspect ratio', Selected),
    function(Index: int): int => Index, new W.Options(Custom := true));
end function;

end unit;
```

## Options and defaults

| Option | Default and behavior |
| --- | --- |
| `Ratios` | An empty array uses 1:1, 4:5, 3:2, 16:9, 9:16 and 21:9. Supply up to 32 ratios with positive dimensions. An empty ratio label shows its proportions. |
| `Custom` | False. Set true to add a final choice at the index after the last ratio. |
| `CustomLabel` | An empty value displays Custom. |
| `Base` | Zero uses 1080 pixels for the short side. A negative value hides the size. |
| `Display` | An empty value shows the size or the selected Custom caption. Supply text to override it. |
| `Plain` | False. Set true to omit the well background and border. |
| `Enabled` | True. Set false to disable input. |

## Interaction

Click a shape to select it. Press Tab to focus a choice. Press Left or Right to
move the selection, or press Enter or Space to select the focused choice.

The change callback receives a zero-based index when the selection changes.
Store the proposed index in the model. Handle the Custom choice in your
application to offer custom dimensions.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_aspect_picker)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/AspectPickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [AspectPicker.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/AspectPicker.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.AspectPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.AspectPicker.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.AspectPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.AspectPicker.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [aspect_picker.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/aspect_picker.c)
  - [aspect_picker.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/aspect_picker.h)
