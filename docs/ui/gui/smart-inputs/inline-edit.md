# Inline edit

The gallery edits the Q3 roadmap title and its owner in place.
Available in **GUI**.

![Roadmap title and owner in the FPAS gallery](images/inline-edit.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.InlineEdit;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Inputs.InlineEdits as W,
  System as Sys, System.Text as Text;

// Displays the confirmed title as an inline editor.
// ## Parameters
// - `Title`: Single-line model text fitting the UTF-8 capacity.
// ## Returns
// An editor that sends the committed title.
public function View(Title: string): E.Element<string>;
requires Title.Length <= 127;
begin
  if Text.Encoding.UTF8.GetByteCount(Title) > 127 or Title.Contains(#10) or Title.Contains(#13) then
    raise new Sys.ArgumentException('The title must fit 127 UTF-8 bytes on one line.');
  end if;

  return W.InlineEdit<string>('roadmap-title', Title,
    function(Proposed: string): string => Proposed,
    new W.Options(Placeholder := 'Add a title', Face := W.Font.Display,
      FontSize := 26.0F, MaxUtf8Bytes := 127));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Placeholder` | Empty. Shown when the model value is empty. |
| `Face / FontSize` | Medium / 0. The other faces are Display and Mono. Zero uses the native body size. |
| `MaxUtf8Bytes` | 255. Set 1 to 511 bytes, excluding the terminator. |
| `Width / Enabled` | Content / true. |

## Interaction

Click the text to edit it. CUI retains the draft until Enter or focus loss
proposes a commit. Escape restores the model text. Handle the committed
proposal in the update and return the accepted title to the view.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/InlineEdits.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/Editors.fpas)
- [Text field](../inputs/text-field.md)
- [Smart inputs](README.md)
