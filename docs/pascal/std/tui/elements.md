# `Std.Tui` elements and identities

`TuiElement` is a data-carrying enum. Its implemented variants are:

```pascal
TuiElement.Empty
TuiElement.Label(Text)
TuiElement.Button(Value)
TuiElement.Input(Value)
TuiElement.TextArea(Id, Text, Caret, Offset, ChangeAction)
TuiElement.CheckBox(Id, Text, Checked, ChangeAction)
TuiElement.List(Id, Items, Selected, ChangeAction)
TuiElement.Scroll(Id, Offset, ChangeAction, Children)
TuiElement.MenuBar(Items)
TuiElement.Menu(Id, Nodes, State, ChangeAction)
TuiElement.StatusLine(Items)
TuiElement.CellGrid(Value)
TuiElement.Rule(Text)
TuiElement.Gauge(Value)
TuiElement.Row(Children, Spacing)
TuiElement.Column(Children, Spacing)
TuiElement.Layout(Settings, Children)
TuiElement.Spacer(Value)
TuiElement.Panel(Title, Children)
TuiElement.Overlay(Title, Children)
TuiElement.Desktop(Focused, Children)
```

`MakeRow` and `MakeColumn` use spacing `0`; use `MakeRowSpaced` and
`MakeColumnSpaced` for gaps. `MakeLayout` and `MakeScroll` wrap exactly one
child. `MenuBar` is the simple flat action bar and uses `TuiMenuItem`.
`Menu` is the controlled hierarchical menu described in
[Menus](menus.md); `StatusLine` uses `TuiStatusItem`. `MakeHint(Text)` paints
ordinary status text, while `MakeKeyHint(KeyText, Text)` paints a display-only
shortcut and its description with separate semantic roles. `MakeCommand`
creates the focusable, actionable status item. Status lines containing only
hints and key hints are not focusable.

`Button(Value)` stores a `TuiButtonSpec` containing its identity, text,
one-character mnemonic, action, default state, and enabled state. Prefer the
builders:

- `MakeButton` creates an enabled ordinary button without a mnemonic;
- `MakeButtonWithMnemonic` creates an enabled ordinary button;
- `MakeDefaultButton` creates the action selected by Enter when no focused
  control handles Enter;
- `MakeDisabledButton` creates a non-focusable, non-actionable button.

The mnemonic must occur in the button text, ignoring case. Buttons are compact
one-row controls. Focus, default state, disabled state, and mnemonics are
expressed through semantic colors instead of block shadows or directional
markers.

`Input(Value)` stores a `TuiInputSpec` with controlled text and caret, optional
hint text, optional oldest-to-newest history, a restorable history draft, and
the source/action identities. Prefer:

- `MakeInput` for a plain controlled input;
- `MakeInputWithHint` for placeholder text shown while the value is empty;
- `MakeHistoryInput` when Up/Down should traverse explicit history.

One-line input content starts one cell inside the field, `◄` and `►` mark hidden
text, and the viewport follows the caret. A focused input paints a block cursor.
Home/End move to the complete
text bounds; Ctrl+Left/Ctrl+Right move by space-delimited words;
Ctrl+Backspace/Ctrl+Delete remove one such word. Up/Down remains unhandled for
plain inputs. For history inputs, Up moves toward older entries and Down toward
newer entries, finally restoring `HistoryDraft`.

`MakePanel` creates ordinary bordered content. `MakeOverlay` creates fixed,
centered modal content when placed directly under `Desktop`. The last overlay
is the active key, menu, focus, and pointer subtree. Overlays are intentionally
not movable and do not paint shadows.

`MakeCellGrid` accepts a flat row-major `TuiCellGrid`. It is the escape hatch
for truecolor dashboards, plots, and visualizations while keeping layout and
terminal lifecycle inside `Std.Tui`. `MakeRule` paints a thin separator with
optional text. `MakeGauge` paints a label, bounded bar, and percentage;
validation requires `Maximum > 0` and `0 <= Value <= Maximum`.

`Scroll` has a one-cell minimum on each axis while retaining its child's
preferred content size. A constrained parent can therefore create a viewport
smaller than the complete content.

Interactive variants require typed control and action identities.
`TuiControlId.Create` and `TuiAction.Create` require positive values. Repeated
action identities are valid, while every control identity must be unique in one
rendered tree.

Before every frame, validation rejects forged non-positive identities,
duplicate control identities, invalid or missing button mnemonics, invalid
input or text-area carets, invalid list selections, negative text-area or
scroll offsets, and focus identities absent from the tree.

Controlled messages are `TextChanged`, `TextAreaChanged`, `CheckChanged`,
`SelectionChanged`, `ScrollChanged`, and `MenuChanged`.
`TuiFocusFirst` and `TuiFocusResolve` choose a valid focus from the active modal
or full tree.

## See also

- [`Std.Tui`](README.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Application routing](application.md#headless-frame-and-routing-order)
