# Voice orb

The gallery includes CUI's legacy voice orb and toggles it between Idle
and Listening.
Available in **GUI**.

![Idle voice orb in the FPAS gallery](images/voice-orb.png)

## Example

```pascal
unit Docs.Ui.Voice.VoiceOrb;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Audio.VoiceOrbs as W, Fpas.Ui.Widgets.Audio.Voice as V;

// Shows the current state of a voice control.
// ## Parameters
// - `Mode`: Current voice activity.
// - `Level`: Normalized current level from zero through one.
// ## Returns
// The orb with an activation message.
public function View(Mode: V.Activity; Level: float): E.Element<string>;
begin
  return W.VoiceOrb<string>('voice-orb', Mode, Level, function(): string => 'toggle-listening');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Activity / Level` | Supply the activity and the latest normalized level. |
| `Activate / Enabled` | Supply a callback to handle activation or pass null for a passive orb. Enabled defaults to true. |
| `Size / Color` | 0 / Automatic. Zero uses the native diameter. |

## Interaction

Pass the newest activity and level to update the orb. With an activation
callback, use its message to handle the change in the application.

![Listening voice orb in the FPAS gallery](images/voice-orb-listening.png)

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Audio/VoiceOrbs.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Voice/View.fpas)
- [Voice](README.md)
