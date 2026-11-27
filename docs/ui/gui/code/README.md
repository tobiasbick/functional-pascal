# Code

The gallery shows source and structured data, edits text and bytes, compares
revisions and displays a build log. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Code block](code-block.md) | Show and copy highlighted source. |
| [JSON viewer](json-viewer.md) | Inspect a document by folding objects and arrays. |
| [Markdown](markdown.md) | Render notes with links and task checkboxes. |
| [Code editor](code-editor.md) | Edit highlighted text and report the caret position. |
| [Diff viewer](diff-viewer.md) | Inspect changes in split or inline form. |
| [Hex viewer](hex-viewer.md) | Inspect or overwrite bytes, mark ranges and jump to offsets. |
| [Log console](log-console.md) | Show timestamped messages with filtering and follow controls. |

## Code gallery page

The original page places the C counter in a full-width code block. Python
and shell share a row with a 16-unit gap and two equal growth weights. The
JSON section shows one nested document and one parse error.

The SQL sample and the editing tools follow the original sections in the
FPAS gallery.

## See also

- [UI handbook](../README.md)
- [Editing tools](../editing-tools/README.md)
- [Terminal](../files/terminal.md)
- [Static code examples in the gallery](../../../../examples/ui/gui/gallery/Pages/Code/View.fpas)
- [Interactive code examples in the gallery](../../../../examples/ui/gui/gallery/Pages/Code/Tools.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#code-evidence)

## Source files

- [Samples.fpas](../../../../examples/ui/gui/gallery/Pages/Code/Samples.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Code/State.fpas)
- [SourceToolsContract.fpas](../../../../tests/Fixtures/gui-host/SourceToolsContract.fpas)
