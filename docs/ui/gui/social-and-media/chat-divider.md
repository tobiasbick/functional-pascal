# Chat divider

Use a chat divider to label a break between messages. The gallery marks
the start of today's conversation.
Available in **GUI**.

![Today divider in the FPAS gallery](images/chat-divider.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.ChatDivider;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Conversation.ChatDividers as W;

// Marks a new day in a chat transcript.
// ## Parameters
// - `DateLabel`: Visible date label.
// ## Returns
// The divider to place between bubbles.
public function View(DateLabel: string): E.Element<string>;
begin
  return W.ChatDivider<string>('day-boundary', DateLabel);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id / Label` | Supply a stable ID and a literal label. |

## Interaction

Insert the divider in the transcript at a day boundary. Format the date
in the application and use distinct IDs for multiple dividers.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Conversation/ChatDividers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/View.fpas)
- [Chat message](chat-message.md)
- [Social and media](README.md)
