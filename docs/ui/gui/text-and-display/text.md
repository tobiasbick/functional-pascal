# Text

The gallery shows a heading, body copy, label, secondary text, muted text
and inline code.
Available in **GUI**.

![Six native text roles in the FPAS gallery](images/text.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Text;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Display.Texts as W;

// Shows a heading with an inline command hint.
// ## Returns
// Native text elements in a column.
public function View(): E.Element of string;
begin
  return E.Column of string(
    W.Text of string('Quickstart', W.Appearance.HeadingOne),
    E.Row of string(
      W.Text of string('Run', W.Appearance.Body),
      W.Text of string('fpas run', W.Appearance.Code)
    )
  );
end function;

// Adds a theme-provided display title like the Overview heading.
// ## Parameters
// - `Caption`: The visible heading.
// ## Returns
// A label styled with the graphical display title role.
public function Title(Caption: string): E.Element of string;
begin
  return E.Styled of string(E.Text of string(Caption), S.Role.DisplayTitle);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Role` | Pass Body, HeadingOne through HeadingFour, Label, Secondary, Muted or Code. |
| `Settings` | `Texts.Options` defaults to inherited reading direction and Start alignment. |

## Usage

Use Body for wrapping paragraphs and Code for a compact monospaced span.
HeadingOne through HeadingFour use `cui_heading` levels 1 through 4.
Secondary uses the body size and the secondary color from `cui_secondary`.
Muted uses the small text size and the muted color from `cui_muted`.
Place the code span in a row with the surrounding text. For a label shared
between hosts, use `Elements.Text`.

Pass `Texts.Options` to fix a text node's reading direction or use `Auto` to
resolve it from the text. Start and End use the resulting direction. Left and
Right select physical edges, and Center centers within the text's bounds.

Apply `Elements.Styled` to a label when its appearance should follow a
semantic role from the theme. Overview uses `DisplayTitle` for its main
heading and `Introduction` for the paragraph beneath it.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_text)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Display/Texts.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Display/View.fpas)
- [Code block](../code/code-block.md)
- [Formatted text](formatted-text.md)
- [Text lowering](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Display/NativeText.fpas)
- [Overview](../overview/README.md)
- [Chapter index](README.md)
- [Terminal example](../../tui/text-and-display/text.md)

## Source files

- [NativeMethods.Texts.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Display/NativeMethods.Texts.cs)
- [CuiWindow.Texts.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Display/CuiWindow.Texts.cs)
- [texts.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/texts.c)
- [texts.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/display/texts.h)
