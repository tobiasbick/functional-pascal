# Rolling number

The gallery changes between two formatted currency totals. Only changed
digits roll.
Available in **GUI**.

![Rolling currency example in the FPAS gallery](images/rolling-number.png)

## Example

```pascal
unit Docs.Ui.Motion.RollingNumber;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Motion.Motion as M;
uses Fpas.Ui.Widgets.Motion.NumberRolls as W;

// Displays a preformatted total.
// ## Parameters
// - `Total`: Formatted amount with any prefix or grouping.
// ## Returns
// The native rolling figure.
public function View(Total: string): E.Element of string;
begin
  return W.NumberRoll of string('total', Total,
    new M.TextOptions(Font := M.Font.Display, Size := 26.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Text` | Format the value in the application. CUI draws tabular digits. |
| `Font / Size / Color` | Body / 0 / Automatic. Zero uses the theme's body size. Automatic uses the theme foreground. |

## Interaction

Keep the ID stable when the text changes. Changed digits roll up when the
value rises and down when the value falls. The width follows the formatted
figure. CUI draws values beyond its animation capacity as complete tabular text.

![Rolling number after changing the value in the FPAS gallery](images/rolling-number-changed.png)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_number_roll)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Motion/NumberRolls.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/View.fpas)
- [Counter](../social-and-media/counter.md)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Motion/`
  - [NumberRoll.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Motion/NumberRoll.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Motion/`
  - [NativeMethods.NumberRoll.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Motion/NativeMethods.NumberRoll.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Motion/`
  - [CuiWindow.NumberRoll.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Motion/CuiWindow.NumberRoll.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/motion/`
  - [number_roll.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/number_roll.c)
  - [number_roll.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/number_roll.h)
