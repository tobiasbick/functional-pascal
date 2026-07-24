# `Std.Tui` controlled text area

`TextArea` is a multiline controlled element. The application model owns its
complete state and supplies it again on every `View`:

```pascal
TuiElementBuilders.MakeTextArea(
  Id,
  Text,
  Caret,
  Offset,
  ChangeAction
)
```

`Id` and `ChangeAction` must be positive typed identities. `Caret` is a
zero-based `Std.Str` character index in the inclusive range
`0..Std.Str.Length(Text)`. `Offset.X` is a non-negative terminal display column;
`Offset.Y` is a non-negative zero-based logical line.

## Controlled change

Keyboard and pointer routing propose the entire next value:

```pascal
TuiMsg.TextAreaChanged(Source, Action, Text, Caret, Offset)
```

Use `TuiMsgTextAreaChanged` to construct the same message explicitly. An
application normally accepts a routed proposal in `Update`:

```pascal
TuiMsg.TextAreaChanged(Source, Action, Text, Caret, Offset):
begin
  return record
    Text := Text;
    Caret := Caret;
    Offset := Offset;
  end
end
```

The host stores no editable text, caret, or scroll state between frames.

## Editing and navigation

When focused, the element handles:

| Key | Proposal |
| --- | --- |
| Character / Space | Insert at the caret. |
| Enter | Insert LF and move to the new line. |
| Tab | Insert two spaces. |
| Backspace / Delete | Delete the preceding or following character. |
| Left / Right | Move one `Std.Str` character. |
| Up / Down | Move one logical line, preserving the nearest terminal display column. |
| Home / End | Move to the start or end of the current logical line. |
| PageUp / PageDown | Move by the arranged viewport height. |

Modified character keys using Ctrl, Alt, or Meta remain unhandled application
keys. After an edit or movement, routing adjusts `Offset` only when needed to
keep the resulting caret visible. An already-visible caret preserves the
model-provided offset.

Tab continues normal focus traversal when another element is focused. Escape
continues to produce `QuitRequested`.

## Pointer and painting

A left-button down inside the arranged viewport focuses the text area and maps
the pointer's row and terminal column through the current offset to the nearest
caret. The resulting `TextAreaChanged` retains the current text and proposes the
new caret and visible offset.

Painting splits text at LF, applies `Offset`, and clips every row and column to
the arranged bounds. A focused visible caret is painted as `▏` with the
`Focused` semantic style role. Unfocused text uses the `Normal` role and does
not paint the caret.

## See also

- [`Std.Tui`](README.md)
- [Elements and identities](elements.md)
- [Layout](layout.md)
- [Application routing](application.md#headless-frame-and-routing-order)
