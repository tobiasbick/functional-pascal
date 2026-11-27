# Fader

Use a fader to change a channel gain. The gallery shows six vertical mixer
strips and a dB scale on Kick. Each strip also displays two signal peaks.
Available in **GUI**.

![Mixer faders in the FPAS gallery](images/fader.png)

## Example

```pascal
unit Docs.Ui.Audio.Fader;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.Faders as W;

// Adjusts the Kick channel and shows its current signal peaks.
// ## Parameters
// - `Gain`: Current gain between minus 48 and plus 6 dB.
// - `Peaks`: Linear amplitudes for the two channels.
// ## Returns
// The fader, with each proposed gain as an application message.
public function View(Gain: float; Peaks: array of float): E.Element of float;
begin
  return W.Fader of float('kick', 'Kick', Gain, -48.0F, 6.0F,
    function(Value: float): float => Value,
    new W.Options(UnitLabel := 'dB', Length := 180.0F,
      Ticks := [6.0F, 0.0F, -6.0F, -12.0F, -24.0F, -48.0F], Levels := Peaks));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value / Minimum / Maximum` | Supply finite values with increasing bounds. |
| `Step / Reset` | 0 / 0. Zero step permits continuous input. Reset uses the bounds and step. |
| `Format` | null. The native readout displays one decimal place. Use the shared numeric format to change the readout. |
| `Length / Horizontal` | 160 logical units / false. Zero length also uses the native default. |
| `Ticks / Levels` | Empty arrays. Supply up to 32 ticks in range and up to eight linear channel peaks. |
| `UnitLabel / HideValue / Enabled` | Empty / false / true. The unit follows the numeric readout. |

## Interaction

Click or drag along the track. Hold Shift while dragging for finer movement.
Use the wheel or arrow keys while the fader has focus. Shift+arrow and Page
Up or Page Down move ten increments. Home and End select the bounds.
Double-click, Delete or Backspace applies `Reset`.

Store the proposed gain in the model. Supply updated channel peaks as the
signal changes. If the lower bound is negative, the meter uses the fader
bounds as its dB scale. Otherwise the meter uses −60 to +6 dB.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_fader)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/Faders.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)
- [Level meter](level-meter.md)
- [Numeric value formats](../charts/README.md#value-format)
- [Audio](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [Fader.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/Fader.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.Fader.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.Fader.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.Fader.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.Fader.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [fader.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/fader.c)
  - [fader.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/fader.h)
