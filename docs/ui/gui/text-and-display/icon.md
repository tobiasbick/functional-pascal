# Icon

The gallery draws a file glyph from the CUI icon font.
Available in **GUI**.

![File icon in the FPAS gallery](images/icon.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Icon;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Display.IconGlyphs as Glyphs;
uses Fpas.Ui.Widgets.Display.Icons as W;
uses Fpas.Ui.Widgets.Theming.Themes as T;

// Draws the native file icon.
// ## Returns
// An 18-point icon in the active accent color.
public function View(): E.Element of string;
begin
  return W.Icon of string(Glyphs.FileText, 18.0F, ThemeColor := T.ColorToken.Accent);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Glyph` | A named glyph from `IconGlyphs` or a string containing a glyph from the active icon face. |
| `Size` | Logical size. Zero uses the native default of 16. |
| `Color` | Optional straight-alpha RGBA. Zero alpha keeps the icon's space with no visible ink. |
| `ThemeColor` | Optional semantic color resolved from the current theme. Examples include `Accent`, `TextMuted` and `Warning`. |

Choose either `Color` or `ThemeColor`. If both are omitted, the icon uses the
theme foreground. `IconGlyphs` includes all the `CUI_ICON_*` constants from the
vendored CUI header.

## Usage

Pair the icon with a label if it carries information. Use an icon button
when the glyph needs to trigger an action.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Display/Icons.fpas)
- [Named CUI glyphs](../../../../src/Fpas.Ui/Widgets/Display/IconGlyphs.fpas)
- [Semantic color tokens](../../../../src/Fpas.Ui/Widgets/Theming/Themes.fpas)
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
- [Rendering regression](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiIconTests.cs)
