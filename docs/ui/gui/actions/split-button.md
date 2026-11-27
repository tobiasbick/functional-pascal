# Split button

Put the primary action and related commands on the same control. Available in **GUI**.

![Split button in the FPAS gallery](images/split-button.png)

## Example

```pascal
unit Docs.Ui.Actions.SplitButton;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.SplitButtons as S;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element of string;
begin
  return S.SplitButton of string('save', 'Save', 'save', [
    new S.Item of string('draft', 'Save as draft', 'draft'),
    new S.Item of string('publish', 'Save and publish', 'publish')
  ]);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Variant` | Primary. Use a native button variant such as Outline. |
| `ControlSize` | Medium. Small and Large are also available. |
| `Icon` | Empty. Provide a native glyph for the main action. |
| `Items` | Supply ordered commands with unique keys, literal labels, and messages. Each `Item.Icon` is optional. |
| `Enabled` | true. Disabling the button closes its menu and blocks both halves. |

## Interaction

Click the main action to send its message. Click the chevron to open or close
the menu. Choose an item to send its message and close the menu.

Tab visits the main action and the chevron separately. Enter and Space activate
the focused half. Down also opens the menu from the chevron. Use Up or Down to
highlight a command and Enter to choose it. Escape closes the menu.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_split_button)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/SplitButtons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/SplitButton.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [SplitButton.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/SplitButton.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.SplitButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.SplitButton.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.SplitButton.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.SplitButton.Types.cs)
  - [CuiWindow.SplitButton.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.SplitButton.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [split_button.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/split_button.c)
  - [split_button.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/split_button.h)
