# Voice waveform

The gallery shows a scrolling live strip and a separate clip with a seek position.
Available in **GUI**.

![Live and seekable waveforms in the FPAS gallery](images/voice-waveform.png)

## Example

```pascal
unit Docs.Ui.Voice.VoiceWaveform;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.Voice as V;
uses Fpas.Ui.Widgets.Audio.VoiceWaves as W;

// Shows a recorded clip with a seek position.
// ## Parameters
// - `Signal`: Clip levels and optional speaker tags.
// - `Progress`: Fraction played from zero through one.
// ## Returns
// The waveform with proposed seek fractions.
public function View(Signal: V.Signal; Progress: float): E.Element of float;
begin
  return W.VoiceWave of float('voice-clip', Signal, Progress,
    function(Next: float): float => Next, new W.Options(Clip := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Signal.Levels` | Supply normalized levels from zero to one in chronological order. Use `Voice.Silent` for a gap. |
| `Signal.Owners` | `null`. Supply one byte tag from `Voice.Owner` per level to color shared speech. |
| `Progress` | `null` omits the playhead. A fraction from zero to one fits the whole clip and colors the played region. |
| `Clip` | `false` displays the newest live levels at the right. Set `true` to fit the whole clip. |
| `Width` | `Fill`. `Content` also uses the available width. Supply a fixed or percentage extent for another width. |
| `Pitch` | `0` uses 3.6 logical units between dots. CUI limits a custom pitch to the range 1.5 to 24. |
| `Step` | `0` uses 0.05 of the clip per arrow key press. |
| `Owner` | `You`. Applies when the signal has no speaker tags. |
| `Color` | `Automatic`. An explicit color sets the live dots or the played dots in a clip with progress. |
| `Enabled` | `true`. Set `false` to disable seeking. |

## Interaction

Pass `null` for the callback to display a read-only strip. Supply a progress
fraction and callback to accept pointer and keyboard seeks. Left and Right
seek by `Step`. Home and End seek to the start and end of the clip.
Apply the proposed fraction to your clip's playback state.

## Resampling

Call `Fpas.Ui.Gui.Cui.CuiWindow.ResampleVoice(Levels, Columns)` to keep a smaller
waveform with a recording. The method uses `cui_voice_resample` without a window.
Each output column takes the highest level in its stretch. A stretch with only
negative or NaN levels returns `Voice.Silent`. An empty signal returns silence in
every column. Pass zero columns to get an empty array.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_voice_wave)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/VoiceWaves.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Voice/View.fpas)
- [Voice note](voice-note.md)
- [Voice session](voice-session.md)
- [Voice](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [VoiceWave.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/VoiceWave.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.VoiceWave.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.VoiceWave.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.VoiceWave.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.VoiceWave.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [voice_wave.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/voice_wave.c)
  - [voice_wave.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/voice_wave.h)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [voice_wave.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/voice_wave.c)
- `tests/Fpas.Ui.Gui.Cui.Tests/Media/`
  - [CuiVoiceResampleTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Media/CuiVoiceResampleTests.cs)
