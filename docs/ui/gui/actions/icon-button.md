# Icon button

Use a glyph for a compact, familiar action. Give it a tooltip that names the action. Available in **GUI**.

![Icon button in the FPAS gallery](images/icon-button.png)

## Example

```pascal
unit Docs.Ui.Actions.IconButton;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.IconButtons as B;
uses System as Sys;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element<string>;
begin
  return B.IconButton<string>('share', Sys.Char.ConvertFromUtf32($E156),
    'Share', 'share');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Variant / ControlSize` | Ghost / Medium |
| `Enabled / Loading / Selected` | true / false / false |
| `Shortcut` | Empty |

## Interaction

Tab focuses the button. Click, Enter or Space sends its message. Busy
buttons keep focus and suppress activation. The GUI uses the CUI icon font.
Shortcut displays a hint.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/IconButtons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Buttons/View.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [IconButton.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/IconButton.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.IconButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.IconButton.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.IconButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.IconButton.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [icon_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/icon_button.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [icon_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/icon_button.c)
  - [icon_button.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/icon_button.h)
