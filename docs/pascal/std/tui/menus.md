# `Std.Tui` menus

`TuiElementBuilders.MakeMenu` creates one controlled hierarchical menu. The
menu is a leaf in the element tree; its hierarchy is a flat array of
`TuiMenuNode` values linked by `Parent: option of TuiMenuNodeId`. Menu nodes
never contain recursive child arrays.

Root nodes must be submenus. Commands, nested submenus, and separators refer to
their parent by identity:

```pascal
var FileId: TuiMenuNodeId := TuiMenuNodeId.Create(1);
var Nodes: array of TuiMenuNode := [
  TuiMenuNodeBuilders.Submenu(FileId, None, 'File', 'F'),
  TuiMenuNodeBuilders.CommandWithShortcut(
    TuiMenuNodeId.Create(2),
    Some(FileId),
    'Open',
    'O',
    TuiAction.Create(10),
    TuiKeyGesture.Create(
      TuiKeyKind.Character, 'o',
      false, true, false, false,
      'Ctrl+O'
    )
  )
];
```

`TuiKeyGesture` stores the key kind, character, modifiers, and display label.
Matching uses the structural fields; the label is only the text painted in the
popup.

## Controlled state

The application model owns `TuiMenuState`. Render it with a stable menu control
and change action:

```pascal
TuiElementBuilders.MakeMenu(MenuId, Nodes, Model.Menu, MenuChangedAction)
```

Routing proposes changes through:

```pascal
TuiMsg.MenuChanged(Source, Action, State)
```

Accept the message by copying `State` into the model. Commands emit
`TuiMsg.Action` with the menu control as `Source`.

`F10`, Alt plus a root mnemonic, Enter, Space, Escape, arrows, and item
mnemonics operate the menu. Enabled command shortcuts are routed globally
inside the active modal subtree. A left-button press opens root menus, follows
submenus, activates commands, or closes an open menu when pressed outside.
While a menu is open, pointer movement switches root popups, selects enabled
commands, and opens nested submenus. Movement outside the menu preserves the
current selection and never activates a command.
Popups are painted as terminal-clamped overlays after the normal desktop
contents. Each popup has a two-column right and one-row bottom shadow painted
with `MenuShadow`.

Validation rejects duplicate or missing node identities, non-submenu parents,
cycles, invalid controlled state, root commands, and duplicate sibling
mnemonics.

## See also

- [`Std.Tui`](README.md)
- [Elements](elements.md)
- [Application](application.md)
- [Cells and themes](cells.md)
