# Chat message

The gallery groups adjacent bubbles and shows a read receipt for the last
outgoing message.
Available in **GUI**.

![Grouped incoming and outgoing chat bubbles in the FPAS gallery](images/chat-message.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.ChatMessage;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Conversation.ChatMessages as W;

// Shows the final outgoing message in a group.
// ## Parameters
// - `Message`: Displayed message body.
// - `Receipt`: Current delivery or read status.
// ## Returns
// The chat bubble for the application view.
public function View(Message: string; Receipt: string): E.Element of string;
begin
  return W.ChatMessage of string('outgoing', Message,
    new W.Options(Mine := true, Grouped := true, LastInGroup := true, Status := Receipt));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Mine` | False. True uses the outgoing accent bubble. |
| `Grouped` | False. True continues the previous group with tighter spacing and suppresses author and time above the bubble. |
| `LastInGroup` | False. True shows the avatar for incoming messages and permits a receipt for grouped outgoing messages. |
| `Author` | Null. Supplies the incoming sender label and avatar initials. |
| `Time` | Null. Supplies the incoming time or the time after an outgoing receipt. |
| `Status` | Null. Supplies the outgoing receipt on an ungrouped or final bubble. |
| `Avatar` | Empty. Pass a local image path to replace the incoming sender's initials. |
| `Markdown` | False. True renders compact Markdown inside the bubble. |
| `OnMarkdown` | Null. With a Markdown body, supply a mapper to enable link and task interaction. |

## Interaction

Set `Grouped` and `LastInGroup` from the message order in the transcript.
Null text options omit the corresponding labels. An empty string keeps the
label or receipt slot with blank text.

The mapper receives the same link, hover and task messages as the standalone
[Markdown widget](../code/markdown.md). Task offsets refer to the UTF-8 body
of the bubble. Handle link activation and persist task changes in the update
function.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_chat_message)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Conversation/ChatMessages.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/View.fpas)
- [Markdown](../code/markdown.md)
- [Chat divider](chat-divider.md)
- [Social and media](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Conversation/`
  - [ChatMessage.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Conversation/ChatMessage.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Conversation/`
  - [NativeMethods.ChatMessage.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Conversation/NativeMethods.ChatMessage.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Conversation/`
  - [CuiWindow.ChatMessage.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Conversation/CuiWindow.ChatMessage.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/conversation/`
  - [chat_message.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/conversation/chat_message.c)
  - [chat_message.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/conversation/chat_message.h)
