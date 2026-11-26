# Now playing

The gallery plays a simulated track with artwork and a seek bar.
Available in **GUI**.

![Low Tide player and transport in the FPAS gallery](images/now-playing.png)

## Example

```pascal
unit Docs.Ui.Motion.NowPlaying;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Audio.NowPlaying as W;

// Shows a model-owned track and transport state.
// ## Parameters
// - `Track`: Track title, artist, playback and artwork.
// ## Returns
// The player with transport and seek proposals.
public function View(Track: W.Value): E.Element<W.Event>;
begin
  return W.NowPlaying<W.Event>('player', Track, function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title / Artist / Position / Duration` | Pass the labels and timing in seconds. |
| `Playing / ArtColor / Artwork` | false / Automatic / empty. Supply a file path for custom artwork. |
| `Width / Compact / Enabled` | Content / false / true. |

## Interaction

Apply the proposed position or play state. Handle previous and next
requests in the application. Supply updated position as playback runs.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/NowPlaying.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/Controls.fpas)
- [Waveform](waveform.md)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Audio/`
  - [NowPlaying.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Audio/NowPlaying.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Audio/`
  - [NativeMethods.NowPlaying.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Audio/NativeMethods.NowPlaying.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Audio/`
  - [CuiWindow.NowPlaying.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.NowPlaying.Types.cs)
  - [CuiWindow.NowPlaying.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Audio/CuiWindow.NowPlaying.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [now_playing.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/now_playing.c)
  - [now_playing.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/now_playing.h)
