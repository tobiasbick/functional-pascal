# Voice

The gallery displays a call, voice messages and a composer in text and
recording modes. All levels, transcript text and playback positions come
from the application. The gallery simulates its audio and transcript.
These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Voice note](voice-note.md) | Present a recorded message with playback and speed controls. |
| [Voice session](voice-session.md) | Show a call timeline, transcript and call controls. |
| [Voice waveform](voice-waveform.md) | Draw live voice levels or seek in a recorded clip. |
| [Voice orb](voice-orb.md) | Show voice activity in an optional activatable indicator. |
| [Composer](composer.md) | Edit a message draft with attachments, model selection or voice recording. |

## See also

- [UI handbook](../README.md)
- [Shared voice types](../../../../src/Fpas.Ui/Widgets/Audio/Voice.fpas)
- [Social and media](../social-and-media/README.md)
- [Voice in the gallery](../../../../examples/ui/gui/gallery/Pages/Voice/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Voice/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Voice/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Voice/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [voice_ai.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/voice_ai.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [ai_voice.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/ai_voice.c)
- `tests/Fixtures/gui-host/`
  - [VoiceAiContract.fpas](../../../../tests/Fixtures/gui-host/VoiceAiContract.fpas)
