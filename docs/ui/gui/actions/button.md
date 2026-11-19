# Button

Run a single action such as saving a document or opening a view. Available in **GUI and TUI**.

![Button in the FPAS gallery](images/button.png)

## Example

```pascal
unit Docs.Ui.Actions.Button;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Actions.Buttons as B,
  Fpas.Ui.Styles as S;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element<string>;
begin
  return B.Button<string>('save', 'Save', 'save',
    new B.Options(Variant := S.ButtonVariant.Primary));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Variant` | Default, which uses the secondary appearance |
| `ControlSize` | Medium. Small and Large are also available. |
| `Enabled / Loading / Selected` | true / false / false |
| `Width` | Content. Fill and Fixed are portable. GUI also accepts Percent. |
| `Icon / IconRight` | Empty. Supply TerminalIcon and TerminalIconRight for terminal glyphs. |

## Interaction

Click, Enter or Space sends the supplied message. Tab focuses the button.
Keep its ID stable when the label or icon changes. Loading keeps focus and
suppresses activation. GUI shows a spinner and TUI shows an ellipsis.

## See also

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
