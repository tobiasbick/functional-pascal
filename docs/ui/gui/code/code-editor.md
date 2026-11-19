# Code editor

Use a code editor to edit source with highlighting and line numbers. It
reports the caret position to the application. The gallery shows a C tile
cache and an editable SQL query.
Available in **GUI**.

![C source editor in the FPAS gallery](images/code-editor.png)

## Example

```pascal
unit Docs.Ui.Code.CodeEditor;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Editors.CodeEditors as W;

// Edits a query stored in the application model.
// ## Parameters
// - `Source`: Current query text.
// ## Returns
// The editor with changes and caret position as messages.
public function View(Source: string): E.Element<W.Event>;
begin
  return W.CodeEditor<W.Event>('sql-editor', Source,
    function(Change: W.Event): W.Event => Change,
    new W.Options(Language := 'sql', Height := 128.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Language / Placeholder` | text / empty. Choose a CUI syntax name such as c or sql. |
| `Width / Height / Size` | Fill / 0 / 0. Zero uses native height or font size. A height of minus one fills the viewport. |
| `Capacity / TabSize / UseTabs` | 65,536 UTF-8 bytes including the terminator / 4 / false. |
| `LineNumbers / ReadOnly` | true / false. ReadOnly allows selection and find. |
| `Marks` | Empty. Sorted UTF-16 scalar-boundary ranges with colors zero (accent) through six. |

## Interaction

Store the new source when `Event.Text` has a value. An event may also
report only caret/anchor, focus, line count or native find results. Cursor and
anchor are UTF-16 scalar-boundary offsets. Lines and visual columns start at
one; tabs count to their configured stops. `Matches` and `MatchIndex` report
the native search result.
`Refused` reports an edit that exceeded the UTF-8 capacity. Replacing the
source from outside the editor resets its undo history.

Return [GUI editor commands](../host.md#code-editor-commands)
from an update to find, navigate or select a range. CUI owns the editing and
search implementation. The [companion example](../../../../examples/ui/gui/editor-commands/README.md)
also shows global shortcuts and custom highlights.

![SQL editor in the FPAS gallery](images/code-editor-sql.png)

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Editors/CodeEditors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Code/Tools.fpas)
- [Code block](code-block.md)
- [Code](README.md)

## Source files

<details>
<summary>Browse 11 source files</summary>

- `src/Fpas.Ui.Gui/Internal/Widgets/Editors/`
  - [CodeEditor.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Editors/CodeEditor.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Editors/`
  - [NativeCodeEditor.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Editors/NativeCodeEditor.cs)
  - [NativeMethods.CodeEditor.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Editors/NativeMethods.CodeEditor.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Editors/`
  - [CuiWindow.CodeEditor.Commands.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.CodeEditor.Commands.cs)
  - [CuiWindow.CodeEditor.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.CodeEditor.Types.cs)
  - [CuiWindow.CodeEditor.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.CodeEditor.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [editor_extensions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/editor_extensions.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/editors/`
  - [code_editor.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/code_editor.c)
  - [code_editor.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/code_editor.h)
- `tests/Fixtures/gui-host/`
  - [EditorCommandsContract.fpas](../../../../tests/Fixtures/gui-host/EditorCommandsContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Editors/`
  - [CuiEditorCommandTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Editors/CuiEditorCommandTests.cs)

</details>
