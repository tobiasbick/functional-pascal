# Dialog

Use a dialog for a focused task that blocks input to the underlying view. Available in **GUI**.

![Dialog in the FPAS gallery](images/dialog.png)

## Example

```pascal
unit Docs.Ui.Overlays.Dialog;

uses Fpas.Ui.Elements as E,
  Fpas.Ui.Widgets.Overlays.Dialogs as W;

// Builds the dialog example.
// ## Parameters
// - `Base`: The view below the overlay.
// - `Open`: Current visibility from the application model.
// ## Returns
// The widget for the application view.
public function View(Base: E.Element<string>; Open: bool): E.Element<string>;
begin
  return W.Dialog<string>('edit-profile', Base, E.Text<string>('Review the profile details.'), Open,
    new W.Options('Edit profile', Description := 'Changes are visible to your team.'),
    'close-profile');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title / Description` | Title is required. Description defaults to empty. |
| `Width` | 440 logical points. CUI fits the width to the viewport. |
| `Persistent / CloseButton` | false / true. Persistent suppresses Escape and scrim dismissal. |
| `InitialFocus` | null. Supply the ID of an enabled control inside the dialog to choose where focus starts. |

## Interaction

Escape, the native close button or a press on the scrim sends the dismiss message. A focused text field gets Escape first. Keep the dialog body available during its exit transition. The host restores focus to the preceding control when the dialog closes.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Overlays/Dialogs.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Dialog.fpas)
- [Action footer](action-footer.md)
- [Overlays](README.md)
