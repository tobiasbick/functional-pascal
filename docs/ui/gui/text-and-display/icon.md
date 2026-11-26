# Icon

The gallery draws a file glyph from the CUI icon font.
Available in **GUI**.

![File icon in the FPAS gallery](images/icon.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Icon;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Display.Icons as W;
uses System as Sys;

// Draws the native file icon.
// ## Returns
// An 18-point icon in the theme foreground.
public function View(): E.Element of string;
begin
  return W.Icon of string(Sys.Char.ConvertFromUtf32($E0CC), 18.0F);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Glyph` | A glyph from the CUI icon font. |
| `Size / Color` | 0 / null. These use the native size and the theme foreground. |

## Usage

Pair the icon with a label if it carries information. Use an icon button
when the glyph needs to trigger an action.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Display/Icons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Display/View.fpas)
- [Icon button](../actions/icon-button.md)
- [Chapter index](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Display/`
  - [Icon.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Display/Icon.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Display/`
  - [NativeMethods.Icon.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Display/NativeMethods.Icon.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Display/`
  - [CuiWindow.Icon.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Display/CuiWindow.Icon.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/display/`
  - [icon.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/icon.c)
  - [icon.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/icon.h)
