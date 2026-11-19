# Voice note

The gallery shows an incoming voice note and can add an outgoing one from
its simulated recorder.
Available in **GUI**.

![Incoming and outgoing voice notes in the FPAS gallery](images/voice-note.png)

## Example

```pascal
unit Docs.Ui.Voice.VoiceNote;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Audio.VoiceNotes as W;

// Shows a voice note at the current playback position.
// ## Parameters
// - `Levels`: Normalized waveform levels.
// - `Playback`: Current duration, position, speed and playing state.
// ## Returns
// The note with transport and speed proposals.
public function View(Levels: array of float; Playback: W.Value): E.Element<W.Event>;
begin
  return W.VoiceNote<W.Event>('message', Levels, Playback,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Duration / Position` | Seconds. Supply a duration. Position defaults to zero. |
| `Speed / Playing` | 1 / false. Supported rates are 1, 1.5 and 2. |
| `Width / Mine / Enabled` | Content / false / true. Mine moves the bubble to the outgoing side. |

## Interaction

Handle Toggle, Seek and Speed proposals in the model and audio player.
Seek reports seconds. Supply updated Position during playback. The
gallery simulates playback by advancing the position on its clock.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Audio/VoiceNotes.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Voice/View.fpas)
- [Composer](composer.md)
- [Gallery playback update](../../../../examples/ui/gui/gallery/Pages/Voice/Update.fpas)
- [Voice](README.md)
