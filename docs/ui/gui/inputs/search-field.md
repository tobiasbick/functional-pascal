# Search field

Use a search field for a single-line query with a search icon and a built-in clear button. See also the [terminal search field](../../tui/inputs/search-field.md).

![Search field in the FPAS gallery](images/search-field.png)

## Example

```pascal
unit Docs.Ui.Inputs.SearchField;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.SearchFields as W;

// Builds the search field example.
// ## Parameters
// - `Query`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Query: string): E.Element of string;
begin
  return W.SearchField of string('input-search', Query, function(Value: string): string => Value,
    new W.Options(Label := 'Search', Placeholder := 'Search projects', MaxUtf8Bytes := 63));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Label / Placeholder / Icon` | Empty. CUI supplies the default search icon. |
| `Hint / Error / Prefix / Suffix` | Empty. Error replaces Hint. Affixes stay outside the query. |
| `ReadOnly / Enabled` | false / true. |
| `ControlSize / MaxUtf8Bytes` | Medium / 255. |
| `OnSubmit` | Optional callback receiving the current query on Enter. |

## Interaction

Edit the query as a single line. The clear button proposes an empty string and returns focus to the field. Use `ClearId(Id)` to address that button in a GUI snapshot. Keep the query in the model and apply it to the results in your update.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_search)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/SearchFields.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/SearchField.fpas)
- [Inputs](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [SearchField.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/SearchField.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.SearchField.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.SearchField.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.SearchField.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.SearchField.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [search_field.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/search_field.c)
  - [search_field.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/search_field.h)
