# Toast

Return a toast command from an update after an operation finishes. Available in **GUI**.

![Toast in the FPAS gallery](images/toast.png)

## Example

```pascal
unit Docs.Ui.Overlays.Toast;

uses Fpas.Ui.Gui.Toasts as W,
  Fpas.Ui.Mvu.Application as Mvu;

// Builds the command for the update function.
// ## Returns
// A command that queues one toast in the GUI host.
public function Notify(): Mvu.Command<string>;
begin
  return W.Show<string>('Changes saved',
    new W.Options(Description := 'Your profile is up to date.', Appearance := W.Tone.Success));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title / Description` | Title is required. Limits are 127 and 191 UTF-8 bytes respectively. |
| `Appearance` | Neutral. Info, Success, Warning and Danger are also available. |
| `Duration` | 0 uses the default of four seconds, or six with the timer line. A negative duration keeps the toast open. |
| `ActionLabel / ShowTimer` | Empty / false. See Undo toast for an action example. |

## Interaction

The host shows the toast after committing the model and leaves focus in place. Hover to expand the stack and pause its timer. Use the dismiss button or swipe right to close a toast.

## See also

- [Widget API and types](../../../../src/Fpas.Ui.Gui/Toasts.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Toast.fpas)
- [Undo toast](../disclosure/undo-toast.md)
- [Overlays](README.md)

## Source files

- [Toast.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Toast.fpas)
- [ToastContract.fpas](../../../../tests/Fixtures/gui-host/ToastContract.fpas)
