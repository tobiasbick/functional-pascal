# Kanban

Use Kanban to group cards by status and let users reorder them. The gallery
composes card bodies from badges, text, progress bars and avatars.
Available in **GUI**.

![The first three columns of the gallery's Kanban board](images/kanban.png)

## Example

```pascal
unit Docs.Ui.Planning.Kanban;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Planning.Kanbans as W;

// Shows the current board.
// ## Parameters
// - `Columns`: Model-owned columns and ordered cards.
// ## Returns
// The board with actions and moves as application messages.
public function View(Columns: array of W.Column<W.Event>): E.Element<W.Event>;
begin
  return W.Kanban<W.Event>('board', Columns, function(Change: W.Event): W.Event => Change);
end function;

// Creates a card with a plain text body.
// ## Parameters
// - `Key`: Stable nonnegative identity, unique on the board.
// - `Title`: Card caption.
// ## Returns
// A card to place in a column.
public function Card(Key: int; Title: string): W.Card<W.Event>;
begin
  return new W.Card<W.Event>(Key, Title, E.Text<W.Event>(Title));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Card.Key / Content` | Use a stable nonnegative key unique to the board and supply a body element. |
| `Column.Dot / Limit / Width` | Automatic / 0 / 0. Zero Limit hides the work-in-progress limit. Zero Width uses the native column width. |
| `Enabled` | true. |

## Interaction

Drag cards within or across columns. With a card focused, Space picks it
up, the arrows move it, and Enter drops it. Escape cancels the move. Apply
`Event.Move` by removing the card first and then inserting it at the proposed
index. The add button and card activation send separate messages. Scroll
horizontally to reach columns outside the visible area.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Planning/Kanbans.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Planning/Board.fpas)
- [Gallery update function](../../../../examples/ui/gui/gallery/Pages/Planning/Update.fpas)
- [Planning](README.md)
