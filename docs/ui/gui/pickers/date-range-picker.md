# Date range picker

Use a date range picker for a start date and an end date. The first gallery field shows relative presets, and the second uses a calendar alone. Available in **GUI**.

![Date range picker in the FPAS gallery](images/date-range-picker.png)

## Example

```pascal
unit Docs.Ui.Pickers.DateRangePicker;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Pickers.DateRangePickers as W;
uses Fpas.Ui.Widgets.Pickers.DateRanges as R;

// Builds the date range picker example.
// ## Parameters
// - `Period`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Period: R.Value): E.Element of R.Value;
begin
  return W.DateRangePicker of R.Value('picker-date-range', Period,
    function(Value: R.Value): R.Value => Value,
    new W.Options(Width := L.Extent.Fixed(L.Measure.Logical(300))));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value` | DateRanges.Value.Empty, Started(Start) or Complete(Start, Finish). Complete endpoints are inclusive and chronological. |
| `Label` | Null omits the label wrapper. An empty string keeps the wrapper without a caption. |
| `Placeholder` | Null shows `Pick a date range`. An empty string hides the prompt. |
| `Width` | Content. CUI prefers 280 logical units without a label. A label wrapper fills the available width by default. Fill, fixed width and percentage also use the native layout. |
| `Presets` | True. False omits the list of relative dates. |
| `Enabled` | True. False prevents input. |

## Interaction

Click the field or activate it with Enter or Space. Choose the first day to
propose `Started` and keep the popup open. Choose the other endpoint in either
order to propose a complete range and close the popup. Escape closes the popup
and restores trigger focus.

Tab moves through the available presets and days. Enter activates the focused
control. Presets such as Today and Last 7 days propose both endpoints at once.
CUI fits one or two months to the available width. In a narrow viewport the
presets move above the calendar.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_date_range_picker)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Pickers/DateRangePickers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/DateRange.fpas)
- [Date range calendar](date-range-calendar.md)
- [Pickers](README.md)

## Source files

- `src/Fpas.Ui/Widgets/Pickers/`
  - [DateRanges.fpas](../../../../src/Fpas.Ui/Widgets/Pickers/DateRanges.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Pickers/`
  - [DateRangeParts.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/DateRangeParts.fpas)
  - [DateRangePicker.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Pickers/DateRangePicker.fpas)
- `src/Fpas.Ui.Gui.Cui/native/widgets/pickers/`
  - [date_range_picker.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/date_range_picker.c)
  - [date_range_picker.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/pickers/date_range_picker.h)
- `tests/Fixtures/gui-host/`
  - [DateRangeContract.fpas](../../../../tests/Fixtures/gui-host/DateRangeContract.fpas)
