# Text field

Edit one line of model-owned text. The gallery lets the user type a track title and add it to the play queue.

![Track title with the caret inside Midnight on the Rhine](../../../../examples/ui/tui/gallery/text-field.png)

## Example

```pascal
unit Docs.Tui.Widgets.TextField;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.TextFields as W;

// Builds the text field example.
// ## Parameters
// - `Title`: Current track title.
// ## Returns
// The resulting view.
public function View(Title: string): E.Element<string>;
begin
  return W.TextField<string>('track-title', Title, function(Value: string): string => Value,
    new W.Options(Label := 'Track title', Placeholder := 'Enter a track title', MaxUtf8Bytes := 120));
end function;

end unit;
```

## Interaction

Click to place the caret. Left/Right and Backspace/Delete move or delete by grapheme. Home/End reach the ends and the line scrolls to keep the caret visible. Paste turns line breaks into spaces and respects the UTF-8 byte limit. Enter invokes an optional `OnSubmit` with the current value. Read-only fields accept focus, disabled fields skip it. The terminal supports labels, hints, errors and the default size.

## See also

- [Inputs](README.md)
- [Shared API](../../widgets.md#text-field)
- [GUI text field](../../gui/inputs/text-field.md)
- [Gallery source](../../../../examples/ui/tui/gallery/QueueSample.fpas)
