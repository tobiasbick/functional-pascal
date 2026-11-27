# Button

Run a single action such as saving a document or opening a view. Available in **GUI and TUI**.

![Button in the FPAS gallery](images/button.png)

## Example

```pascal
unit Docs.Ui.Actions.Button;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Actions.Buttons as B;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element of string;
begin
  return B.Button of string('save', 'Save', 'save',
    new B.Options(Variant := S.ButtonVariant.Primary));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Variant` | Default uses Secondary on its own and Ghost in a GUI button group. |
| `ControlSize` | Medium. Small and Large are also available. |
| `Enabled / Loading / Selected` | true / false / false |
| `Width` | Content. Use Fill for the available width, or Fixed for an explicit width. GUI also accepts Percent. |
| `Icon / IconRight` | Empty. Supply TerminalIcon and TerminalIconRight for terminal glyphs. |
| `Tooltip / Shortcut` | Empty. GUI displays the shortcut hint with a nonblank caption. Exclude the middle-dot delimiter with two spaces on each side. Both strings must fit 255 UTF-8 bytes, including six separator bytes for the shortcut. |
| `Color` | null. The button variant supplies the foreground. |

Use `Elements.Size` with a width growth weight to allocate a weighted share of a
GUI row. [Sizing and units](../../layout/sizing.md#independent-gui-axes)

## Interaction

Click, Enter or Space sends the supplied message. Tab focuses enabled buttons.
Keep the ID stable when the label or icon changes. During loading, the button
keeps its label and focus and suppresses activation. GUI shows a spinner.
TUI shows an ellipsis. Set `Selected` for the selected appearance.
Bind the shortcut key separately.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_button)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/Buttons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Buttons/View.fpas)
- [Actions](README.md)
- [Terminal example](../../tui/actions/button.md)

## Source files

- [Shared gallery example](../../../../examples/ui/gui/gallery/Shared/ButtonExamples.fpas)

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [Button.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/Button.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.Button.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.Button.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.Button.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.Button.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [button.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/button.c)
  - [button.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/button.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Actions/`
  - [CuiButtonTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Actions/CuiButtonTests.cs)
