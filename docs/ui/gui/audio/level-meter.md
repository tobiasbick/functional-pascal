# Level meter

Use a level meter for signal peaks and clipping. The gallery draws the stereo
master as two horizontal bars with segmented fill and a dB scale.
Available in **GUI**.

![Stereo level meter in the FPAS gallery](images/level-meter.png)

## Example

```pascal
unit Docs.Ui.Audio.LevelMeter;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.LevelMeters as W;

// Displays a two-channel master meter.
// ## Parameters
// - `Left`: Linear peak of the left channel.
// - `Right`: Linear peak of the right channel.
// ## Returns
// The meter with a message for a cleared clip indicator.
public function View(Left: float; Right: float): E.Element of string;
begin
  return W.LevelMeter of string('master', [Left, Right],
    new W.Options(Length := 240.0F, Horizontal := true, Scale := true, Segmented := true),
    function(): string => 'clip-cleared');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Levels` | One to eight finite, nonnegative linear peaks. 1.0 represents 0 dBFS. |
| `MinimumDb / MaximumDb` | −60 / +6 dBFS. Zero uses the respective default. The resulting minimum must be negative and less than the maximum. |
| `Length / Thickness` | 160 / 6 logical units. Zero uses the respective default. |
| `Horizontal` | False. Horizontal bars increase from left to right. Vertical bars increase from bottom to top. |
| `Scale` | False. A scale labels the preset dB marks that fit within the selected range and length. |
| `Segmented` | False. Set true for LED segments. |
| `Hold / Decay` | 1.5 seconds / 20 dB per second. Zero uses the respective default. A negative hold removes the delay before the peak mark falls. |

## Interaction

Pass fresh peaks as frames update. The bars rise to each peak and fall at
the selected decay rate. The level colors change at −18 and −6 dBFS.

Each channel latches its clip indicator at 0 dBFS. Click the clip lights to
clear all latched indicators and send `OnClipCleared` if supplied. A peak
that still reaches 0 dBFS lights its indicator again. If a channel leaves
the list, the meter discards its hold and clip state.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_level_meter)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/LevelMeters.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)
- [Audio](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [LevelMeter.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/LevelMeter.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.LevelMeter.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.LevelMeter.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.LevelMeter.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.LevelMeter.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [level_meter.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/level_meter.c)
  - [level_meter.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/level_meter.h)
- `tests/Fpas.Ui.Gui.Cui.Tests/Feedback/`
  - [CuiLevelMeterTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Feedback/CuiLevelMeterTests.cs)
