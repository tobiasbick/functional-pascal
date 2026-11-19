# Icon

The gallery draws a file glyph from the CUI icon font.
Available in **GUI**.

![File icon in the FPAS gallery](images/icon.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Icon;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Display.Icons as W, System as Sys;

// Draws the native file icon.
// ## Returns
// An 18-point icon in the theme foreground.
public function View(): E.Element<string>;
begin
  return W.Icon<string>(Sys.Char.ConvertFromUtf32($E0CC), 18.0F);
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
