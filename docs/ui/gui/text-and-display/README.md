# Text and display

The gallery uses native text roles and glyphs, a rounded image and a row
of clickable swatches. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Text](text.md) | Show headings, labels, body copy and inline code. |
| [Formatted text](formatted-text.md) | Draw a label with an explicit face, size and color. |
| [Icon](icon.md) | Draw a glyph from the CUI icon font. |
| [Emoji](emoji.md) | Draw an emoji sequence alongside a label or count. |
| [Keycap](keycap.md) | Show the keys for a shortcut. |
| [Image](image.md) | Display a file or RGBA pixels with optional rounded corners. |
| [Interactive canvas](canvas.md) | Draw shapes and receive clicks in local coordinates. |

## See also

- [UI handbook](../README.md)
- [Link](../disclosure/link.md)
- [Avatar](../disclosure/avatar.md)
- [Text and display in the gallery](../../../../examples/ui/gui/gallery/Pages/Display/View.fpas)

## Source files

- [display.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/display.c)
- [DisplayContract.fpas](../../../../tests/Fixtures/gui-host/DisplayContract.fpas)
