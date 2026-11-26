# Status bar

Place short pieces of state and small actions along the bottom of a pane. A spacer pushes trailing items to the right.

![Saved status with encoding, indentation and language on one row](../../../../examples/ui/tui/gallery/statusbar.png)

## Example

```pascal
unit Docs.Tui.Widgets.StatusBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Desktop.StatusBars as W;

// Builds the status bar example.
// ## Parameters
// - `Encoding`: Current document encoding.
// ## Returns
// The resulting view.
public function View(Encoding: string): E.Element of string;
begin
  return W.StatusBar of string('status', [
    (W.Item of string).Text(new W.Label('Saved')), (W.Item of string).Spacer,
    (W.Item of string).Action('encoding', new W.Label(Encoding), 'Change encoding', true, 'encoding')
  ], true);
end function;

end unit;
```

## Interaction

Under **Extended → App chrome**, tab to the encoding, indentation or
language and use Enter to cycle its value. The saved state is plain text.
Text, spacers and inline progress do not take focus. Place a growing
content area above the bar to keep it at the bottom as the terminal resizes.

## See also

- [App chrome](README.md)
- [Shared API](../../widgets.md#desktop-chrome)
- [GUI status bar](../../gui/app-chrome/status-bar.md)
- [Gallery source](../../../../examples/ui/tui/gallery/ToolSamples.fpas)

## Source files

- [StatusBar.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Desktop/StatusBar.fpas)
