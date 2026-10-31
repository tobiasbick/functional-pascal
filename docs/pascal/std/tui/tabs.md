# Tabs

`TuiElementBuilders.MakeTabs(Bar, Page)` places a model-controlled tab bar on
one side of the supplied active page. `TuiTabPosition.Top`, `Bottom`, `Left`,
and `Right` choose its side and orientation. The application constructs only
the selected page and keeps the selection and all page drafts in its model.
Vertical tab bars leave one column between the bar and the page.
Hidden pages therefore do not receive focus or input.

```pascal id=tabs run
program TabExample;
uses Std.Ui.Terminal as Tui;
begin
const Bar := Tui.TuiTabBar(
  Id := Tui.TuiControlId.Create(1),
  Items := [Tui.TuiTabItem(Text := 'World', Enabled := true),
    Tui.TuiTabItem(Text := 'Gamemaster', Enabled := true)],
  Selected := 0,
  Position := Tui.TuiTabPosition.Top,
  ChangeAction := Tui.TuiAction.Create(2)
);
discard Tui.TuiElementBuilders.MakeTabs(Bar, Tui.TuiElementBuilders.MakeLabel('World settings'));
end.
```

Selection is a zero-based index. If at least one item is enabled, selection
must name an enabled item. Empty or entirely disabled bars use `-1` and are
not focusable. A change emits `TuiMsg.SelectionChanged(Id, ChangeAction,
Selected)`; the application accepts it in its update function. The bar's
control and action IDs must be positive, and its control ID must be unique.

When focused, Left/Right navigate horizontal bars and Up/Down navigate vertical
bars. Navigation wraps and skips disabled items; Home/End select the first or
last enabled item. A left-button press selects an enabled tab. Overflow markers
show hidden items and can be clicked to move to the preceding or next enabled
tab. The selected item remains visible as far as the available cells permit;
long labels are clipped at terminal-cell boundaries. Tab traverses between the
bar and the controls in its active page; Shift-Tab traverses in reverse. The usual
text-area Tab behavior still applies.

`TabNormal`, `TabSelected`, `TabSelectedFocused`, and `TabDisabled` are optional
palette overrides. Their default fallbacks are `Normal`, `Selected`, `Focused`,
and `Disabled`, respectively. The selected style remains visible when focus moves
into the page.
