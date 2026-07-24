# `Std.Tui` elements and identities

`TuiElement` is a data-carrying enum. Its implemented variants are:

```pascal
TuiElement.Empty
TuiElement.Label(Text)
TuiElement.Button(Id, Text, Action)
TuiElement.Input(Id, Text, Caret, ChangeAction)
TuiElement.TextArea(Id, Text, Caret, Offset, ChangeAction)
TuiElement.CheckBox(Id, Text, Checked, ChangeAction)
TuiElement.List(Id, Items, Selected, ChangeAction)
TuiElement.Scroll(Id, Offset, ChangeAction, Children)
TuiElement.MenuBar(Items)
TuiElement.StatusLine(Items)
TuiElement.Row(Children, Spacing)
TuiElement.Column(Children, Spacing)
TuiElement.Layout(Settings, Children)
TuiElement.Spacer(Value)
TuiElement.Window(Title, Children)
TuiElement.Dialog(Title, Children)
TuiElement.Desktop(Focused, Children)
```

`MakeRow` and `MakeColumn` use spacing `0`; use `MakeRowSpaced` and
`MakeColumnSpaced` for gaps. `MakeLayout` and `MakeScroll` wrap exactly one
child. `MenuBar` uses `TuiMenuItem`; `StatusLine` uses `TuiStatusItem`.
Hint-only status lines are not focusable.

Interactive variants require typed control and action identities.
`TuiControlId.Create` and `TuiAction.Create` require positive values. Repeated
action identities are valid, while every control identity must be unique in one
rendered tree.

Before every frame, validation rejects forged non-positive identities, duplicate
control identities, invalid input or text-area carets, invalid list selections,
negative text-area or scroll offsets, and focus identities absent from the
tree.

Controlled messages are `TextChanged`, `TextAreaChanged`, `CheckChanged`,
`SelectionChanged`, and `ScrollChanged`. `TuiFocusFirst` and `TuiFocusResolve`
choose a valid focus from the active modal or full tree.

## See also

- [`Std.Tui`](README.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Application routing](application.md#headless-frame-and-routing-order)
