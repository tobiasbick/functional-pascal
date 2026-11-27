# Uptime bars

Use uptime bars for a service status page. Each service has a row with one bar
per day, the latest status and the average across days with data.
Available in **GUI**.

![Service availability by day in the FPAS gallery](images/uptime.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.Uptime;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.Uptimes as W;
uses System as Sys;

// Displays the latest availability history.
// ## Parameters
// - `Services`: Service names and daily availability, oldest first.
// - `LastDay`: Date corresponding to the newest sample.
// ## Returns
// The availability panel for the application view.
public function View(Services: array of W.Service; LastDay: Sys.DateOnly): E.Element of string;
begin
  return W.Uptime of string('availability', Services, LastDay);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Service.Name` | Supply the name shown above the daily bars. |
| `Service.Days` | Supply fractions from zero through one, oldest first. Negative samples represent missing data. |
| `LastDay` | Null uses relative hover labels. Supply a date to label the days with calendar dates. |
| `Width` | Fill. |
| `Enabled` | True. Set to false to suppress input. |
| `Hovered` | Null. To receive hover changes, pass the last reported bar and a `Changed` callback. |

## Interaction

Hover a bar to see its date and availability. Narrow panels retain the most
recent days that fit.

Initialize `Settings.Hovered` with `new W.Hover()` to track changes. Save the
`Hover` value from `Changed` in the model and pass it to the next view. Both
indices are zero-based. `Hover.Day` indexes the full service history. Both
indices are minus one when the pointer leaves the bars.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_uptime)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/Uptimes.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [More developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [Uptime.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/Uptime.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.Uptime.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.Uptime.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.Uptime.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Uptime.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [uptime.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/uptime.c)
  - [uptime.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/uptime.h)
