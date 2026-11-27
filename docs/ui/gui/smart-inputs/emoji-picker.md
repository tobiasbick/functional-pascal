# Emoji picker

Smart inputs opens a native emoji palette beside the message field.
Available in **GUI**.

![Open emoji picker beside the message field in the FPAS gallery](images/emoji-picker.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.EmojiPicker;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Actions.IconButtons as B;
uses Fpas.Ui.Widgets.Inputs.TextFields as F;
uses Fpas.Ui.Widgets.Overlays.Menus as M;
uses Fpas.Ui.Widgets.Overlays.Popovers as P;
uses Fpas.Ui.Widgets.Pickers.EmojiPickers as W;
uses System as Sys;

// Messages for the field and its emoji palette.
public type Msg = variant
  // Accepts a text edit.
  Edit(Value: string);
  // Opens or dismisses the popover.
  SetOpen(Open: bool);
  // Applies a palette selection or skin tone change.
  ChangeEmoji(Change: W.Change);
end variant;

// Builds the message field and emoji popover.
// ## Parameters
// - `Text`: Current message text.
// - `Open`: Whether the popover is visible.
// - `Preferences`: Model-owned recent emoji and skin tone.
// ## Returns
// A row containing the field and anchored palette.
public function View(Text: string; Open: bool; Preferences: W.State): E.Element of Msg;
begin
  const Trigger: E.Element of Msg := B.IconButton of Msg(
    'emoji-trigger', Sys.Char.ConvertFromUtf32($E164), 'Emoji', Msg.SetOpen(not Open));
  const Picker: E.Element of Msg := W.EmojiPicker of Msg(
    'message-emoji', Preferences, Msg.ChangeEmoji, new W.Options(Height := 260.0F));
  return E.Row of Msg(
    new L.Flow(Gap := L.Spacing.Fixed(L.Measure.Logical(8)), Align := L.Alignment.Center),
    F.TextField of Msg('message', Text, Msg.Edit),
    P.Popover of Msg('emoji-popup', Trigger, [Picker], Open, Msg.SetOpen(false),
      new P.Options(Position := M.Placement.BottomEnd, Width := 340.0F))
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable picker ID. |
| `State.Recent` | Empty model-owned history. Separate emoji with spaces and list the newest first. Pass null for CUI-owned history. |
| `RecentCapacity` | 256 bytes, including the terminating byte. Applies when the application owns the history. |
| `State.Tone` | 0. The tones range from 0 to 5. Pass null for CUI-owned tone state. |
| `Width` | `Content` fills the available width up to 360 logical units. Use `Fill`, a fixed logical width or a percentage to set a different extent. |
| `Height` | Grid height. Zero uses 280 logical units. |
| `Size` | Emoji size. Zero uses 26 logical units. |
| `Placeholder / EmptyText / Hint` | Null uses CUI's default captions: `Search emoji`, `No emoji found`, and `Pick an emoji`. |
| `GroupNames` | Null uses CUI's names. Supply ten captions to rename the tabs and group headings. |
| `Search / Preview / ToneSelector / Enabled` | All true. Hide individual controls or disable input. |

## Interaction

Search by name or use a group tab. Supply `GroupNames` in this order:
Recent, Smileys & emotion, People & body, Animals & nature, Food & drink,
Travel & places, Activities, Objects, Symbols, Flags.

Store `Change.Value` to retain model-owned history and tone. A model-owned
tone change has a null `Change.Emoji`. A selected emoji still reaches the
application when it exceeds the recent-history capacity. CUI-owned values
stay null in proposals.

The gallery keeps the popover open when the user selects a tone. Selecting
an emoji appends it and closes the popover if the result fits the field's
UTF-8 capacity.

![Message field before the emoji palette opens](images/emoji-message.png)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_emoji_picker)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/EmojiPickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/EmojiMessage.fpas)
- [Gallery update](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Update.fpas)
- [Popover](../overlays/popover.md)
- [Emoji display](../text-and-display/emoji.md)
- [Smart inputs](README.md)

## Source files

- [CuiEmojiPickerTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiEmojiPickerTests.cs)
- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [EmojiPicker.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/EmojiPicker.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Pickers/`
  - [NativeMethods.EmojiPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Pickers/NativeMethods.EmojiPicker.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Pickers/`
  - [CuiWindow.EmojiPicker.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.EmojiPicker.Types.cs)
  - [CuiWindow.EmojiPicker.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Pickers/CuiWindow.EmojiPicker.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [emoji_picker.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/emoji_picker.c)
  - [emoji_picker.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/emoji_picker.h)
- `tests/Fixtures/gui-host/`
  - [ColorEmojiContract.fpas](../../../../tests/Fixtures/gui-host/ColorEmojiContract.fpas)
