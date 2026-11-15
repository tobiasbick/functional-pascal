# Filter bar

Use a filter bar for search and multiple choices organized by field. The
gallery starts with Engine and Platform selected in Team. Available in **GUI**.

![Team filter chip and search field in the FPAS gallery](images/filter-bar.png)

## Example

```pascal
unit Docs.Ui.DataGrid.FilterBar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Collections.FilterBars as W;

// Shows filters for the people view.
// ## Parameters
// - `Current`: Model-owned masks and search text.
// ## Returns
// The bar with each proposed filter state as an application message.
public function View(Current: W.Value): E.Element<W.Value>;
begin
  const Fields: array of W.Field := [
    new W.Field('Team', ['Engine', 'Design', 'Platform']),
    new W.Field('Status', ['Active', 'Away', 'Invited'])
  ];
  return W.FilterBar<W.Value>('people-filters', Fields, Current,
    function(Value: W.Value): W.Value => Value);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Field.Values / Icon` | Up to 32 values per field. The CUI glyph defaults to empty. |
| `Value.Selected` | Supply one unsigned mask per field. Bit zero corresponds to the first value. A zero mask clears the field. |
| `Value.Search` | Pass an empty string to show an empty search box or null to hide it. |
| `SearchCapacity / Enabled` | 256 UTF-8 bytes including the terminator / true. |

## Interaction

Use Filter to add a field, open its chip to edit the selected values, and use
Clear all to reset the bar. A change proposes new masks and search text. In
your update, store the value and apply the criteria to the data you display.
For the example, `new W.Value([5u, 0u], '')` selects Engine and Platform.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Collections/FilterBars.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/DataGrid/View.fpas)
- [Data grid](README.md)
