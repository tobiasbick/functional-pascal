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
TuiElement.Menu(Id, Nodes, State, ChangeAction)
TuiElement.StatusLine(Items)
TuiElement.Row(Children, Spacing)
TuiElement.Column(Children, Spacing)
TuiElement.Layout(Settings, Children)
TuiElement.Spacer(Value)
TuiElement.Window(Title, Children)
TuiElement.Dialog(Title, Children)
TuiElement.MovableDialog(Id, Title, Position, DragOffset, ChangeAction, Children)
TuiElement.Desktop(Focused, Children)
```

`MakeRow` and `MakeColumn` use spacing `0`; use `MakeRowSpaced` and
`MakeColumnSpaced` for gaps. `MakeLayout` and `MakeScroll` wrap exactly one
child. `MenuBar` is the simple flat action bar and uses `TuiMenuItem`.
`Menu` is the controlled hierarchical menu described in
[Menus](menus.md); `StatusLine` uses `TuiStatusItem`.
Hint-only status lines are not focusable.

`MakeDialog` creates a centered fixed modal. `MakeMovableDialog` creates a
controlled modal with a model-owned optional position and drag offset. A
position of `None` centers the dialog. Pressing its title bar, dragging with the
left button, and releasing emit `DialogChanged`; the application stores the
proposed position and drag offset and returns them from its next `View`.
Movable dialogs are clamped so their frame and two-column, one-row shadow remain
on the desktop whenever the terminal is large enough.

`Scroll` has a one-cell minimum on each axis while retaining its child's
preferred content size. A constrained parent can therefore create a viewport
smaller than the complete content.

Interactive variants require typed control and action identities.
`TuiControlId.Create` and `TuiAction.Create` require positive values. Repeated
action identities are valid, while every control identity must be unique in one
rendered tree.

Before every frame, validation rejects forged non-positive identities, duplicate
control identities, invalid input or text-area carets, invalid list selections,
negative text-area or scroll offsets, and focus identities absent from the
tree.

Controlled messages are `TextChanged`, `TextAreaChanged`, `CheckChanged`,
`SelectionChanged`, `ScrollChanged`, `MenuChanged`, and `DialogChanged`.
`TuiFocusFirst` and `TuiFocusResolve` choose a valid focus from the active modal
or full tree.

## See also

- [`Std.Tui`](README.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Application routing](application.md#headless-frame-and-routing-order)
