# Status tag

The gallery shows seven native status presets and a row of compact tags that users can remove.
Available in **GUI**.

![Status presets in the FPAS gallery](images/status-tag.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.StatusTag;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Feedback.StatusTags as W;

// Displays a removable review status.
// ## Parameters
// - `Label`: Caption, or empty for the preset caption.
// ## Returns
// The status tag with a removal message.
public function View(Label: string): E.Element<string>;
begin
  return W.StatusTag<string>('review', new W.Value(Label, W.Status.InReview),
    function(): string => 'remove-review', new W.Options(Small := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value.Label / Preset` | Empty / Pending. An empty label uses the preset caption. |
| `Small / Icon / Tone` | false / empty / Neutral. Neutral keeps the preset tone. |
| `Remove / Enabled` | null / true. Supply a callback to make the tag removable. |

## Interaction

For a custom state, supply a label, icon and tone with `Status.Custom`.
Handle the removal message in the model.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/StatusTags.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)
