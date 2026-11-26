# Composer

Use a composer for text or voice messages. The Voice gallery switches
from a text draft to a recorder with levels and elapsed time.
Available in **GUI**.

![Message composer with microphone action in the FPAS gallery](images/composer.png)

## Example

```pascal
unit Docs.Ui.Voice.Composer;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.Composers as W;

// Shows a message composer with optional recording.
// ## Parameters
// - `Draft`: Current draft, attachments, model options and optional recording.
// ## Returns
// The composer with draft and action proposals.
public function View(Draft: W.Value): E.Element<W.Event>;
begin
  return W.Composer<W.Event>('message', Draft, function(Change: W.Event): W.Event => Change,
    new W.Options(Placeholder := 'Message', Microphone := true, MaxUtf8Bytes := 255));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value.Draft / Attachments / Models` | Empty text and empty arrays. Attachments are displayed file names. Models supplies the available choices. |
| `Selected / Busy / Take` | 0 / false / null. Busy shows Stop. Set Take to show recording controls. |
| `Width / Placeholder / MaxLines / MaxUtf8Bytes` | Fill / empty / 0 / 511. Zero MaxLines uses the native limit. The byte limit excludes the terminator. |
| `Microphone / Enabled` | false / true. Microphone adds the record action in text mode. |

## Interaction

Apply draft edits and handle Send, Stop, attachment and model actions in
the application. On Record, create a take and supply its levels and
elapsed seconds. Handle Cancel by discarding the take and SendVoice by
submitting it. The gallery supplies synthetic recorder levels.

![Composer showing a simulated voice recording in the FPAS gallery](images/composer-recording.png)

The AI page supplies a model menu and attached file names in the same composer.

![Composer with attachments and a model menu in the FPAS gallery](images/composer-ai.png)

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/Composers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Voice/View.fpas)
- [Voice note](voice-note.md)
- [AI model picker](../ai/model-picker.md)
- [Gallery recording update](../../../../examples/ui/gui/gallery/Pages/Voice/Update.fpas)
- [Voice](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [Composer.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/Composer.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.Composer.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.Composer.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.Composer.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.Composer.Types.cs)
  - [CuiWindow.Composer.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.Composer.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [composer.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/composer.c)
  - [composer.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/composer.h)
