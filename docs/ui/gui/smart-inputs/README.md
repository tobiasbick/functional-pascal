# Smart inputs

The gallery edits a heading in place, adjusts numbers by dragging, formats
amounts, records shortcuts and inserts emoji into a message. These widgets
require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Inline edit](inline-edit.md) | Edit a title or other single-line value in place. |
| [Scrub field](scrub-field.md) | Adjust a bounded number by dragging or typing. |
| [Money field](money-field.md) | Edit an amount with a currency symbol and code. |
| [Shortcut recorder](shortcut-recorder.md) | Capture a key combination and display it as keycaps. |
| [Emoji picker](emoji-picker.md) | Browse or search emoji and insert a selection. |
| [Text field affixes](../inputs/text-field.md#prefix-and-suffix) | Show a fixed prefix or unit beside editable text. |

## Smart inputs gallery page

The title and owner share a native column with a gap of six logical units.
The scrub fields, affix fields and recorders use wrapping rows. The Money
section places two 240-unit boxes in one row. The amounts use exact decimals.

The message and its emoji trigger share a fill row with a maximum width of
480 logical units. The message accepts 159 UTF-8 bytes. The anchored picker
keeps the recent emoji and skin tone in the gallery model.

## See also

- [UI handbook](../README.md)
- [Inputs](../inputs/README.md)
- [Smart inputs in the gallery](../../../../examples/ui/gui/gallery/Pages/SmartInputs/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/SmartInputs/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/SmartInputs/State.fpas)
- `examples/ui/gui/gallery/Pages/SmartInputs/Widgets/`
  - [Editors.fpas](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/Editors.fpas)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.SmartInput.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.SmartInput.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [smart_inputs.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/smart_inputs.c)
- `tests/Fixtures/gui-host/`
  - [CompletionContract.fpas](../../../../tests/Fixtures/gui-host/CompletionContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiSmartInputTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiSmartInputTests.cs)
