# Toggle group

The **Extended → Actions → Toggle group** section shows the CUI gallery's
formatting toggles (**Bold**, **Italic**, **Underline**, **Code**) and exclusive
alignment (**Left**, **Center**, **Right**).

Use `ToggleGroups.ToggleGroup` with one selection flag per item. Tab reaches
the group once, at the first independent toggle or at the selected exclusive
toggle. Left and Right move focus within the group. Home and End reach the
ends. Click, Enter or Space toggles an independent item or selects an
exclusive item. Each change proposes a fresh array of flags.

The terminal displays text labels with selection markers. `ControlSize`
controls padding and row height. Narrow strips scroll to keep focus visible.
Disabled groups skip input. Use `ToggleGroups.ItemId(Id, Index)` to focus an
item or find its `toggle-group-item` snapshot.

## See also

- [Shared API](../../widgets.md#toggle-group)
- [GUI toggle group](../../gui/actions/toggle-group.md)

## Source files

- [ToggleGroup.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Actions/ToggleGroup.fpas)
- [ChoiceSamples.fpas](../../../../examples/ui/tui/gallery/ChoiceSamples.fpas)
