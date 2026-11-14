# Tooltip

Attach a tooltip to one control to explain its action or show a shortcut hint. Available in **GUI**.

![Tooltip in the FPAS gallery](images/tooltip.png)

## Example

```pascal
unit Docs.Ui.Overlays.Tooltip;

uses Fpas.Ui.Elements as E,
  Fpas.Ui.Widgets.Overlays.Tooltips as W;

// Builds the tooltip example.
// ## Parameters
// - `Target`: The control that receives the overlay.
// ## Returns
// The widget for the application view.
public function View(Target: E.Element<string>): E.Element<string>;
begin
  return W.Tooltip<string>(Target, 'Bold', 'Ctrl B');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Text` | A nonblank caption. The caption and shortcut together must fit 255 UTF-8 bytes, including six separator bytes when a shortcut is supplied. |
| `Shortcut` | Empty. Bind the gesture separately. |

## Interaction

CUI opens the tooltip after about 550 ms of hover. Moving directly to another tooltipped control switches the hint immediately while the tooltip is warm. The target keeps its input and layout.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Overlays/Tooltips.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Tooltip.fpas)
- [Overlays](README.md)
