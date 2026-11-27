# Filter bar

Use a filter bar for search and multiple choices organized by field. The
gallery starts with Engine and Platform selected in Team. Available in
**GUI** and [TUI](../../tui/data-display/filter-bar.md).

![Team filter chip and search field in the FPAS gallery](images/filter-bar.png)

## Example

```pascal
unit Docs.Ui.DataGrid.FilterBar;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Collections.FilterBars as W;

// Shows filters for the people view.
// ## Parameters
// - `Current`: Model-owned masks and search text.
// ## Returns
// The bar with each proposed filter state as an application message.
public function View(Current: W.Value): E.Element of W.Value;
begin
  const Fields: array of W.Field := [
    new W.Field('Team', ['Engine', 'Design', 'Platform']),
    new W.Field('Status', ['Active', 'Away', 'Invited'])
  ];
  return W.FilterBar of W.Value('people-filters', Fields, Current,
    function(Value: W.Value): W.Value => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Fields` | Up to 32 fields, in display order. |
| `Field.Name` | Literal caption for the chip and the Filter menu. |
| `Field.Values` | Up to 32 values, in mask bit order. |
| `Field.Icon` | `null` omits the glyph. An empty string keeps the icon slot. |
| `Value.Selected` | One unsigned mask per field. Bit zero selects the first value. A zero mask clears the choices. |
| `Value.Search` | `null` hides the search field. An empty string shows an empty search field. |
| `SearchCapacity` | 256 UTF-8 bytes including the terminator. With search enabled, use 2 through 65,536 bytes. |
| `Enabled` | `true`. |

## Interaction

Use **Filter** to add a field. CUI opens the field's checklist and
keeps its chip until you remove it. Click a value to toggle it while the
checklist remains open. You can use the keyboard to open a chip. Use the
pointer to select its values. [CUI-061](../../../future/cui-upstream-bugs.md#cui-061)

Use the clear button on a chip to remove it. **Clear all** appears when
the search or any mask contains a value. It clears the search and removes
all chips.

The bar proposes new masks and search text. In your update, store the
proposal and apply the criteria to your data. The gallery shows the
controls and does not apply them to the nearby grid. For the example,
`new W.Value([5u, 0u], '')` selects Engine and Platform.

The GUI host reports the visible controls as `<id>/search`, `<id>/add`,
`<id>/clear` and `<id>/chip/<index>`. Chip indices start at zero. A
focus request for the bar selects Search if present, then Filter if
present, then the first chip.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_filter_bar)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Collections/FilterBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [Data grid](README.md)

## Source files

- [Shared gallery filter fields and initial selection](../../../../examples/ui/gui/gallery/Shared/PeopleFilters.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Collections/`
  - [FilterBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Collections/FilterBar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Collections/`
  - [NativeMethods.FilterBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Collections/NativeMethods.FilterBar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Collections/`
  - [CuiWindow.FilterBar.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.FilterBar.Types.cs)
  - [CuiWindow.FilterBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Collections/CuiWindow.FilterBar.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/collections/`
  - [filter_bar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/filter_bar.c)
  - [filter_bar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/collections/filter_bar.h)
