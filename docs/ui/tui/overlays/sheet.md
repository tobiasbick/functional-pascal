# Sheet and drawer

Dock a secondary view to an edge while retaining the current page in the background. The gallery uses a right-hand sheet for the play queue.

![Right-hand Play queue sheet listing two tracks and a Done button](../../../../examples/ui/tui/gallery/sheet.png)

## Example

```pascal
unit Docs.Tui.Widgets.Sheet;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Overlays.Sheets as W, Fpas.Ui.Widgets.Actions.Buttons as B;

// Builds the sheet and drawer example.
// ## Parameters
// - `Base`: View below the overlay.
// - `Open`: Model-owned visibility.
// ## Returns
// The resulting view.
public function View(Base: E.Element<string>; Open: bool): E.Element<string>;
begin
  return W.Sheet<string>('queue', Base,
    E.Column<string>(E.Text<string>('1. First light'), B.Button<string>('done', 'Done', 'close')),
    Open, new W.Options('Play queue', Size := 272, InitialFocus := 'done'), 'close');
end function;

end unit;
```

## Interaction

The edge defaults to Right. Size sets the width for a side sheet or height for a top or bottom sheet. Tab stays in the sheet. Page Up and Page Down scroll the body while the header remains visible. Escape or a backdrop click proposes dismissal.

## See also

- [Overlays](README.md)
- [Shared API](../../widgets.md#sheet-and-drawer)
- [GUI sheet and drawer](../../gui/overlays/sheet.md)
- [Gallery source](../../../../examples/ui/tui/gallery/QueueSample.fpas)
