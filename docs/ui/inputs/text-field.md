# Text field

Use a text field for a single line of editable text. The gallery shows a name field and an email field with an icon and hint. Available in **GUI and TUI**.

![Text field in the FPAS gallery](images/text-field.png)

## Example

```pascal
unit Docs.Ui.Inputs.TextField;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.TextFields as W;

// Builds the text field example.
// ## Parameters
// - `Name`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Name: string): E.Element<string>;
begin
  return W.TextField<string>('input-name', Name, function(Value: string): string => Value,
    new W.Options(Label := 'Full name', Placeholder := 'Maya Okafor', MaxUtf8Bytes := 63));
end function;

// Edits the address shown after the HTTPS prefix.
// ## Parameters
// - `Host`: The address without a scheme, up to 95 UTF-8 bytes.
// ## Returns
// A field that sends the edited address without the prefix.
public function Address(Host: string): E.Element<string>;
begin
  return W.TextField<string>('input-url', Host, function(Value: string): string => Value,
    new W.Options(Prefix := 'https://', MaxUtf8Bytes := 95));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Label / Placeholder / Icon` | Empty. Use a label to name the field and a placeholder to suggest a value. |
| `Hint / Error` | Empty. An error replaces the hint. |
| `Prefix / Suffix` | Empty. Affixes stay outside the editable value. |
| `Password / Clearable / ReadOnly / Enabled` | false / false / false / true. |
| `ControlSize / MaxUtf8Bytes` | Medium / 255. The password mode supports a limit of up to 341 bytes. |

## Interaction

In the GUI, use the native caret, selection, clipboard and undo keys while the field has
focus. Enter sends the optional submission message after any edit from that
frame. Replacing the value from the model moves the caret to the end and
clears undo history. A read-only field still allows selection. Password mode
masks the value and supplies a reveal button. GUI snapshots omit the password
value.

## Terminal

Use the same `TextField` with a label, placeholder, hint or error. Click to
place the caret. ←/→ moves by grapheme, Home/End jumps to either end, and
Backspace/Delete removes the previous or next grapheme. A long value scrolls
horizontally as the caret moves. Pasting turns line breaks into spaces and
stops before exceeding `MaxUtf8Bytes`. Enter invokes `OnSubmit` with the
current model value. Tab moves focus to the next control.

A read-only field accepts focus and caret movement. A disabled field skips
focus. Password and clear buttons, icons, affixes and non-default control
sizes require the GUI.

![Editing a track title in the terminal gallery](../../../examples/ui/tui/gallery/text-field.png)

## Prefix and suffix

In Smart inputs, the URL shows an HTTPS prefix and the weight shows a
kilogram suffix. The model stores just the editable address and number.
Build the full URL from the address in the update when you need it.

![URL prefix and weight suffix in the FPAS gallery](images/text-field-affixes.png)

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Inputs/TextFields.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/TextField.fpas)
- [Smart inputs gallery example](../../../examples/ui/gui/gallery/Pages/SmartInputs/View.fpas)
- [Inputs](README.md)
