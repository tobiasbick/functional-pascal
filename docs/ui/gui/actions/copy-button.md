# Copy button

Send literal text to the clipboard and show brief feedback next to the copied value. Available in **GUI**.

![Copy button in the FPAS gallery](images/copy-button.png)

## Example

```pascal
unit Docs.Ui.Actions.CopyButton;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Actions.CopyButtons as C;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element<string>;
begin
  return C.CopyButton<string>('copy-install', 'npm i cui');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `OnCopied` | Absent by default |
| `Enabled` | true |

## Interaction

Click, Enter or Space submits the text. The check mark lasts 1.6 seconds and
focus stays on the button. GUI uses the platform clipboard. The optional callback
reports submission.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Actions/CopyButtons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Buttons/Widgets/CopyButton.fpas)
- [Actions](README.md)
