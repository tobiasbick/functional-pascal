# Pickers

The Pickers gallery page covers multiple choices, date ranges, clock times and
measurements. Each control proposes a new value through its callback. Keep
accepted values in the model and pass them back on the next render. These
widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Multi-select](multi-select.md) | Choose several items from a fixed list. |
| [Date range picker](date-range-picker.md) | Choose start and end dates with optional relative presets. |
| [Date range calendar](date-range-calendar.md) | Keep one or two months visible while choosing a range. |
| [Time picker](time-picker.md) | Select a time in a 24-hour or 12-hour field. |
| [Wheel picker](wheel-picker.md) | Choose a caption by scrolling a list. |
| [Time dial](time-dial.md) | Choose hours and minutes on an analog face. |
| [Ruler slider](ruler-slider.md) | Adjust a measurement against a graduated tape. |
| [Histogram slider](histogram-slider.md) | Select a range over a distribution. |

## Pickers gallery page

The page puts the 320-unit language selector in a row. The two date fields
wrap at their 300-unit widths. A separate card sizes to the inline calendar's
two months. The Stay field and the calendar share one date range.

The time fields wrap together. The date reel uses three joined wheels inside
a card beside the time dial. The final row gives equal available width to the
two rulers and the histogram card.

## See also

- [UI handbook](../README.md)
- [Inputs](../inputs/README.md)
- [Pickers in the gallery](../../../../examples/ui/gui/gallery/Pages/Pickers/View.fpas)
- [CUI gallery comparison](../../../future/cui-compare/README.md#pickers-evidence)

## Source files

- `examples/ui/gui/gallery/Pages/Pickers/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/Update.fpas)
- `examples/ui/gui/gallery/Pages/Pickers/Widgets/`
  - [DateRange.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/DateRange.fpas)
  - [RulerAndHistogram.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/RulerAndHistogram.fpas)
  - [Time.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/Time.fpas)
  - [WheelsAndDial.fpas](../../../../examples/ui/gui/gallery/Pages/Pickers/Widgets/WheelsAndDial.fpas)
- `tests/Fixtures/gui-host/`
  - [ActionsPickersContract.fpas](../../../../tests/Fixtures/gui-host/ActionsPickersContract.fpas)
  - [DateRangeContract.fpas](../../../../tests/Fixtures/gui-host/DateRangeContract.fpas)
  - [PickerFieldsContract.fpas](../../../../tests/Fixtures/gui-host/PickerFieldsContract.fpas)
  - [RulerHistogramContract.fpas](../../../../tests/Fixtures/gui-host/RulerHistogramContract.fpas)
  - [TimePickersContract.fpas](../../../../tests/Fixtures/gui-host/TimePickersContract.fpas)
