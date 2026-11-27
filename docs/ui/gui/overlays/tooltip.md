# Tooltip

Attach a tooltip to one control to explain its action or show a shortcut hint. Available in **GUI**.

![Tooltip in the FPAS gallery](images/tooltip.png)

## Example

```pascal
unit Docs.Ui.Overlays.Tooltip;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Tooltips as W;

// Builds the tooltip example.
// ## Parameters
// - `Target`: The control that receives the overlay.
// ## Returns
// The widget for the application view.
public function View(Target: E.Element of string): E.Element of string;
begin
  return W.Tooltip of string(Target, 'Bold', 'Ctrl B');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Text` | A nonblank caption without the middle-dot delimiter with two spaces on each side. The caption and shortcut together must fit 255 UTF-8 bytes, including six separator bytes when a shortcut is supplied. |
| `Shortcut` | Empty. CUI shows space-separated keys next to the caption. Bind the gesture separately. |

## Interaction

CUI opens the tooltip after about 550 ms of hover. Moving to another control
with a tooltip switches the hint immediately during the native warm interval.
The target keeps its input and layout. CUI suppresses the tooltip during a
pointer press.

Use the built-in tooltip on buttons and icon buttons. The Overlays toolbar uses
the icon buttons' tooltip and shortcut options. [Icon button](../actions/icon-button.md)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_tooltip)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/Tooltips.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Tooltip.fpas)
- [Overlays](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [Tooltip.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/Tooltip.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.Tooltip.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.Tooltip.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.Tooltip.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Tooltip.Types.cs)
  - [CuiWindow.Tooltip.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.Tooltip.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [tooltip.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/tooltip.c)
  - [tooltip.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/tooltip.h)
- `tests/Fixtures/gui-host/`
  - [TooltipContract.fpas](../../../../tests/Fixtures/gui-host/TooltipContract.fpas)
