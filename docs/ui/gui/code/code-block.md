# Code block

Use a code block for read-only source with highlighting and a native copy
button. The gallery includes C, Python, shell and SQL. Available in **GUI**.

![Highlighted C code block in the FPAS gallery](images/code-block.png)

## Example

```pascal
unit Docs.Ui.Code.CodeBlock;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.CodeBlocks as W;

// Displays the current SQL query.
// ## Parameters
// - `Query`: SQL source text.
// ## Returns
// A code block with line numbers and a copy button.
public function View(Query: string): E.Element of string;
begin
  return W.CodeBlock of string('query', Query,
    new W.Options(Language := 'sql', Title := 'people.sql', LineNumbers := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Language` | Null. CUI uses generic highlighting for null and unrecognized values. |
| `Title` | Null. CUI uses the language as the header caption if the title is null. |
| `LineNumbers` | False. When true, CUI numbers lines from one. |
| `HighlightFrom / HighlightTo` | Zero / zero. Set an inclusive range of one-based lines. A zero end highlights only the line at `HighlightFrom`. |
| `MaxHeight` | Zero. Set a positive limit on the source viewport height in logical units. |
| `Size` | Zero. CUI uses its default font size of 13 logical units. |
| `NoHeader` | False. True hides the header and copy button. |

## Syntax languages

Language names are case-sensitive. The header displays the supplied name.

| Language | Names |
| --- | --- |
| C and C++ | `c`, `cpp`, `c++`, `h` |
| JavaScript and TypeScript | `js`, `javascript`, `ts`, `typescript` |
| Python | `python`, `py` |
| Rust | `rust`, `rs` |
| Go | `go` |
| JSON | `json` |
| Shell | `shell`, `sh`, `bash` |
| SQL | `sql` |
| Plain text | `text`, `plain` |

## Interaction

Set either `Title` or `Language` to show the header and copy button. An empty
string supplies the corresponding caption or badge with blank text. If both
values are null, the header is omitted.

Click the copy button or activate it with Enter or Space. The button copies
the complete source with its original whitespace. Use `CopyId(Id)` to
address the button for focus or inspection.

Long lines scroll horizontally. Code taller than `MaxHeight` scrolls
vertically. When the source changes size, the viewport keeps the offset
within the new content bounds. The display uses four spaces for each tab.

![Highlighted SQL code block in the FPAS gallery](images/code-block-sql.png)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_code_block)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/CodeBlocks.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Code/View.fpas)
- [Code editor](code-editor.md)
- [Code](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [CodeBlock.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/CodeBlock.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.CodeBlock.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.CodeBlock.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.CodeBlock.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.CodeBlock.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [code_block.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/code_block.c)
  - [code_block.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/code_block.h)
