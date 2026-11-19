# Emoji picker

Smart inputs opens a native emoji palette beside the message field.
Available in **GUI**.

![Open emoji picker beside the message field in the FPAS gallery](images/emoji-picker.png)

## Example

```pascal
unit Docs.Ui.SmartInputs.EmojiPicker;

uses
  Fpas.Ui.Elements as E, Fpas.Ui.Layout as L,
  Fpas.Ui.Widgets.Inputs.TextFields as F, Fpas.Ui.Widgets.Actions.IconButtons as B,
  Fpas.Ui.Widgets.Pickers.EmojiPickers as W, Fpas.Ui.Widgets.Overlays.Popovers as P,
  Fpas.Ui.Widgets.Overlays.Menus as M, System as Sys;

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
public function View(Text: string; Open: bool; Preferences: W.State): E.Element<Msg>;
begin
  const Trigger: E.Element<Msg> := B.IconButton<Msg>(
    'emoji-trigger', Sys.Char.ConvertFromUtf32($E164), 'Emoji', Msg.SetOpen(not Open));
  const Picker: E.Element<Msg> := W.EmojiPicker<Msg>(
    'message-emoji', Preferences, Msg.ChangeEmoji, new W.Options(Height := 260.0F));
  return E.Row<Msg>(
    new L.Flow(Gap := L.Spacing.Fixed(L.Measure.Logical(8)), Align := L.Alignment.Center),
    F.TextField<Msg>('message', Text, Msg.Edit),
    P.Popover<Msg>('emoji-popup', Trigger, Picker, Open, Msg.SetOpen(false),
      new P.Options(Position := M.Placement.BottomEnd, Width := 340))
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `State.Recent / Tone` | Empty / 0. History is space-separated, newest first, up to 255 UTF-8 bytes. Tone ranges from 0 to 5. |
| `Width / Height / Size` | All zero. CUI chooses the palette width, grid height and glyph size. |
| `Placeholder / EmptyText / Hint` | Empty. Override the search prompt, empty result text and preview hint. |
| `Search / Preview / ToneSelector / Enabled` | All true. |

## Interaction

Search by name or jump to a group. Accept `Change.Value` in the update
to keep history and skin tone. A tone change has a null `Change.Emoji`,
so the gallery keeps the palette open. When an emoji is selected, the
gallery appends it and closes the popover if the resulting message fits
the field's UTF-8 capacity.

![Message field before the emoji palette opens](images/emoji-message.png)

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/EmojiPickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Widgets/EmojiMessage.fpas)
- [Gallery update](../../../../examples/ui/gui/gallery/Pages/SmartInputs/Update.fpas)
- [Popover](../overlays/popover.md)
- [Emoji display](../text-and-display/emoji.md)
- [Smart inputs](README.md)
