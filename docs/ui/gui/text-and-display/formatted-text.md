# Formatted text

App chrome draws the editor sample and line numbers in the native monospace
face. Available in **GUI**.

![Monospace code and muted line numbers in the FPAS gallery](images/formatted-text.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.FormattedText;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Motion.Motion as M;
uses Fpas.Ui.Widgets.Theming.Themes as T;

// Draws one line from the editor preview.
// ## Parameters
// - `LineNumber`: Line number to display at the left.
// - `Content`: The source line.
// - `Colors`: Current theme colors.
// ## Returns
// One row of two explicitly formatted labels.
public function View(LineNumber: int; Content: string; Colors: T.Palette): E.Element<string>;
begin
  return E.Row<string>(
    new L.Flow(Gap := L.Spacing.Fixed(L.Measure.Logical(18))),
    E.FormattedText<string>(LineNumber.ToString(), M.Font.Mono, 12.0F, Colors.TextMuted),
    E.FormattedText<string>(Content, M.Font.Mono, 12.0F, Colors.Foreground)
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Text / Font` | Supply the visible text and the native face, such as Mono. |
| `Size` | Logical font size. Zero uses the theme size. |
| `Color` | Supply a straight-alpha color. Use the current palette to follow the host appearance. |

## Usage

Use this label when the view needs an exact face, size and color. For
semantic headings and captions, use the text roles on the Text page.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Elements.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Desktop/Editor.fpas)
- [Native font choices](../../../../src/Fpas.Ui/Widgets/Motion/Motion.fpas)
- [Text](text.md)
- [Text and display](README.md)

## Source files

- [NativeText.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Display/NativeText.fpas)
