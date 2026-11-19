# Reactions

Show counts and the current user's reaction alongside a message or activity. Available in **GUI**.

![Reactions in the FPAS gallery](images/reactions.png)

## Example

```pascal
unit Docs.Ui.Actions.Reactions;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Conversation.Reactions as R;

// The reaction and its position after activation.
public type Change = record
  // Zero-based position in the displayed list.
  public Index: int;
  // Proposed count and selected state.
  public Item: R.Item;
end record;

// Builds the example view.
// ## Parameters
// - `Items`: Current application value.
// ## Returns
// The widget with its message type.
public function View(Items: array of R.Item): E.Element<Change>;
begin
  return R.Reactions<Change>('votes', Items,
    function(Index: int; Item: R.Item): Change => new Change(Index, Item));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Item.Active` | false |
| `Enabled` | true |
| `Item fields` | Label, CUI glyph and nonnegative count |

## Interaction

Click, Enter or Space toggles a reaction. The mapper gets the zero-based
index and the proposed item. Save that item in the model to apply its count
and active flag. Use ItemId to request focus for a particular reaction.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Conversation/Reactions.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/Reactions.fpas)
- [Actions](README.md)
