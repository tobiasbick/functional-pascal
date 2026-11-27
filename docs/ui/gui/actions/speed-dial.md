# Speed dial

Group closely related commands behind one floating action button. Available in **GUI**.

![Speed dial in the FPAS gallery](images/speed-dial.png)

## Example

```pascal
unit Docs.Ui.Actions.SpeedDial;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Actions.SpeedDials as D;
uses System as Sys;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element of int;
begin
  return D.SpeedDial of int('create', [
    new D.Item('New note', Sys.Char.ConvertFromUtf32($E0CC)),
    new D.Item('Upload image', Sys.Char.ConvertFromUtf32($E0F6))
  ], function(Index: int): int => Index, new D.Options(Tooltip := 'Create'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | One to sixteen actions. A null label omits the label bubble. A null icon omits the glyph. |
| `Icon` | Null uses a plus sign on the closed trigger. An empty string leaves the closed trigger blank. |
| `Tooltip` | Null omits the trigger tooltip. |
| `Position` | `BottomStart` selects CUI's `TopEnd` default. |
| `Enabled` | true. |

## Interaction

Click or press Enter or Space on the trigger to open the dial. Choose a
visible action by pointer or keyboard to send its zero-based index. Press
Escape or click outside to dismiss the dial. Escape from a focused action
returns focus to the trigger. Use `ItemId` to request focus on a visible
action or read its control bounds.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_speed_dial)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/SpeedDials.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/SpeedDial.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Actions/`
  - [SpeedDial.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Actions/SpeedDial.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Actions/`
  - [NativeMethods.SpeedDial.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Actions/NativeMethods.SpeedDial.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Actions/`
  - [CuiWindow.SpeedDial.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.SpeedDial.Types.cs)
  - [CuiWindow.SpeedDial.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Actions/CuiWindow.SpeedDial.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/actions/`
  - [speed_dial.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/speed_dial.c)
  - [speed_dial.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/actions/speed_dial.h)
