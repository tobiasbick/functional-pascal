# `Std.Ui.Terminal` elements and identities

`TuiElement` is a data-carrying enum. Its implemented variants are:

```text
TuiElement.Empty
TuiElement.Label(Value)
TuiElement.Button(Value)
TuiElement.Input(Value)
TuiElement.TextArea(Id, Text, Caret, Offset, ChangeAction)
TuiElement.CheckBox(Id, Text, Checked, ChangeAction)
TuiElement.List(Id, Items, Selected, Actions)
TuiElement.TabBar(Value)
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
TuiElement.Overlay(Spec, Children)
TuiElement.Desktop(Focused, Children);
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
one-character mnemonic, action, default, active, and enabled states. Prefer the
builders:

- `MakeButton` creates an enabled ordinary button without a mnemonic;
- `MakeActiveButton` marks an active choice without assigning a default action;
- `MakeButtonWithMnemonic` creates an enabled ordinary button;
- `MakeDefaultButton` creates the action selected by Enter when no focused
  control handles Enter;
- `MakeDisabledButton` creates a non-focusable, non-actionable button.

The mnemonic must occur in the button text, ignoring case. Buttons are compact
one-row controls with a space of padding on each side and no automatic brackets.
Focus, active and default states, disabled state, and mnemonics are
expressed through semantic colors instead of block shadows or directional
markers.

`MakeButtonOfKind` sets the semantic meaning (`Normal`, `Primary`, `Cancel`,
`Warning`, or `Dangerous`) independently of interaction and default-action
behavior. Themes provide `TuiButtonColors` for each meaning. A button's visual
state is normal, active, focused, pressed, or disabled; an idle default action
has its own style. Applications can also construct a `TuiButtonSpec` directly
to combine a meaning with a default action or disabled state.

`MakeHeading` paints one-line text using the title color of its surrounding
panel or overlay. Ordinary `MakeLabel` text inherits the surrounding surface.

`MakeList` emits a controlled selection change. `MakeActivatingList` additionally
accepts an activation action: Enter activates the focused selection, and a click
selects and activates the clicked item. When both occur, the selection change
arrives before the activation. Pointer activation is delivered only if the next
view still has that list and activation action, with the clicked index and text
still selected inside the active focus scope. Rejecting the proposed selection
or replacing the clicked item does not activate a different row.
A list constrained to fewer rows than items scrolls to keep the selection
visible. The lower-level `TuiListActions` record
contains `Change` and optional `Activate` actions.

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
`MakeResponsiveOverlay` uses the same modal behavior with a viewport-relative
size rule; see [Layout](layout.md#responsive-overlays).

`MakeTabs` places a controlled `TabBar` beside its active page. See
[Tabs](tabs.md) for selection, input, placement, and overflow.

`MakeCellGrid` accepts a flat row-major `TuiCellGrid`. It is the escape hatch
for truecolor dashboards, plots, and visualizations while keeping layout and
terminal lifecycle inside `Std.Ui.Terminal`. `MakeRule` paints a thin separator with
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

- [`Std.Ui.Terminal`](README.md)
- [Text area](text-area.md)
- [Layout](layout.md)
- [Application routing](application.md#headless-frame-and-routing-order)
