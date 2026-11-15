# Keycap

The gallery renders the command-palette hint as three keycaps.
Available in **GUI**.

![Ctrl Shift P keycaps in the FPAS gallery](images/keycap.png)

## Example

```pascal
unit Docs.Ui.TextAndDisplay.Keycap;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Display.Keycaps as W;

// Shows the command-palette shortcut hint.
// ## Returns
// Three native keycaps.
public function View(): E.Element<string>;
begin
  return W.Keycap<string>('Ctrl Shift P');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Keys` | Space-separated key labels. |

## Usage

Keycaps are labels. Register or configure the actual shortcut on the
widget or application action that handles it.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Display/Keycaps.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Display/View.fpas)
- [Command palette](../overlays/command-palette.md)
- [Chapter index](README.md)
