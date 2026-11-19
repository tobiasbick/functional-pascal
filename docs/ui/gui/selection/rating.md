# Rating

Use a rating to collect a whole number of stars. Available in **GUI**.

![Rating in the FPAS gallery](images/rating.png)

## Example

```pascal
unit Docs.Ui.Selection.Rating;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.Ratings as C;

// Builds the rating example.
// ## Parameters
// - `Stars`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Stars: int): E.Element<int>;
begin
  return C.Rating<int>('rating', 'Rating', Stars, function(Value: int): int => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Label` | Name used for control inspection. |
| `Value` | A model-owned integer from zero through Maximum. |
| `Maximum` | 5. Accepts 1 through 16 stars. |
| `Enabled` | true. |
| `StarSize` | 22 logical points. Accepts any finite positive size. |

## Interaction

Click a star to propose its number. Click the selected star again to propose zero. The native rating takes pointer input.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/Ratings.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Selection/Widgets/Rating.fpas)
- [Selection](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [Rating.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/Rating.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.Rating.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.Rating.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.Rating.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.Rating.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [rating.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/rating.c)
  - [rating.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/rating.h)
