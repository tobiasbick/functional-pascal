# Control tile

The gallery shows Wi-Fi, Hotspot, Focus and AirShare, plus a wide
Dark appearance tile.
Available in **GUI**.

![Quick-setting control tiles in the FPAS gallery](images/control-tile.png)

## Example

```pascal
unit Docs.Ui.Motion.ControlTile;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.ControlTiles as W;
uses System as Sys;

// Toggles the Wi-Fi setting.
// ## Parameters
// - `On`: Whether Wi-Fi is enabled in the model.
// ## Returns
// The tile with the proposed state as its message.
public function View(On: bool): E.Element of bool;
begin
  return W.ControlTile of bool('wi-fi', 'Wi-Fi', On, function(Next: bool): bool => Next,
    new W.Options(Icon := Sys.Char.ConvertFromUtf32($E1AE)));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Icon / Detail / Color` | Empty / empty / Automatic. Set Detail to describe the current state. |
| `Width / Wide / Enabled` | Fill / false / true. Wide selects the horizontal layout. |

## Interaction

Activate the tile to propose the opposite state. Store the new value
and update any detail text in the model.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_control_tile)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/ControlTiles.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/Controls.fpas)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [ControlTile.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/ControlTile.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.ControlTile.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.ControlTile.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.ControlTile.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.ControlTile.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [control_tile.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/control_tile.c)
  - [control_tile.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/control_tile.h)
