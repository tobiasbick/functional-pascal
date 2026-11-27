# Inline edit

The gallery edits the Q3 roadmap title and its owner in place.
Available in **GUI**.

![Roadmap title and owner in the FPAS gallery](images/inline-edit.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.InlineEdit;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.InlineEdits as W;
uses System as Sys;
uses System.Text as Text;

// Displays the confirmed title as an inline editor.
// ## Parameters
// - `Title`: Single-line model text fitting the UTF-8 capacity.
// ## Returns
// An editor that sends the committed title.
public function View(Title: string): E.Element of string;
requires Title.Length <= 127;
begin
  if Text.Encoding.UTF8.GetByteCount(Title) > 127 or Title.Contains(#10) or Title.Contains(#13) then
    raise new Sys.ArgumentException('The title must fit 127 UTF-8 bytes on one line.');
  end if;

  return W.InlineEdit of string('roadmap-title', Title,
    function(Proposed: string): string => Proposed,
    new W.Options(Placeholder := 'Add a title', Face := W.Font.Display,
      FontSize := 26.0F, MaxUtf8Bytes := 127));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Placeholder` | Null. CUI shows Untitled at rest when the value is empty. An empty caption hides the placeholder. |
| `Face` | Medium. Display, Mono, and Icons are also available. The same face displays the text at rest and while editing. |
| `FontSize` | 0 uses the native body size. Set a positive size in logical units to override it. |
| `MaxUtf8Bytes` | 255. Set 1 to 511 bytes, excluding the terminator. |
| `Width` | Content fits the text and grows the editor as the user types. Fill, Fixed, and Percent set the outer width. |
| `Enabled` | true. Disabling an editor discards its draft. |

## Interaction

Click the text to select it for editing. Tab focuses the text. Enter or Space
on focused text starts editing. Enter or focus loss proposes a commit if the
draft differs from the model. Escape discards the draft and its undo history.
Handle the committed proposal in the update and return the accepted value.

If the model changes while editing, the editor replaces the draft and resets
the undo history. Use the font roles to customize the selected face for the
window. [Fonts and native captions](../host.md#fonts-and-native-captions)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_inline_edit)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/InlineEdits.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/Editors.fpas)
- [Text field](../inputs/text-field.md)
- [Smart inputs](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [InlineEdit.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/InlineEdit.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.InlineEdit.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.InlineEdit.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.InlineEdit.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.InlineEdit.Types.cs)
  - [CuiWindow.InlineEdit.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.InlineEdit.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [inline_edit.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/inline_edit.c)
  - [inline_edit.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/inline_edit.h)
