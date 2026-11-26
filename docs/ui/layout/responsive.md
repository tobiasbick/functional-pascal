# Responsive views

The host passes a `Layout.Viewport` to a responsive `View`. `Width` and `Height` classify the two axes independently. Read `LogicalWidth` and `LogicalHeight` when the view needs the measured dimensions. A viewport record can omit those measurements.

## Size classes

| Host and axis | `Compact` | `Regular` | `Expanded` |
| --- | --- | --- | --- |
| GUI width in logical units | Below 600 | From 600 to below 1200 | 1200 or more |
| GUI height in logical units | Below 400 | From 400 to below 900 | 900 or more |
| TUI width in columns | Below 60 | From 60 to below 100 | 100 or more |
| TUI height in rows | Below 20 | From 20 to below 35 | 35 or more |

The GUI host reports the drawable dimensions, which can include fractions. The TUI host converts the terminal dimensions to logical units. One column maps to eight logical units and one row maps to 16.

## Choose a layout

Use fill sizing and bounds when the same view fits different sizes. In the GUI, wrapping rows and automatic grid columns adapt to the allocated width. Branch on `Viewport.Width` or `Viewport.Height` to change the structure of a view.

If a cached subview uses dimensions or size classes, include the values in its dependencies. The following view stacks the panes at compact widths.

```pascal
unit Docs.Ui.Layout.Responsive;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;

// Arranges two panes according to the available width.
// ## Parameters
// - `Viewport`: The available size from the host.
// - `First`: The first pane.
// - `Second`: The second pane.
// ## Returns
// A column at compact widths and a row at larger widths.
public function View(
  Viewport: L.Viewport;
  First: E.Element of string;
  Second: E.Element of string
): E.Element of string;
begin
  if Viewport.Width = L.SizeClass.Compact then
    return E.Column of string(First, Second);
  end if;

  return E.Row of string(First, Second);
end function;

end unit;
```

## See also

- [Sizing and units](sizing.md)
- [GUI frame lifecycle and resize](../gui/host.md#invalidation-and-inspection)
- [TUI layout](../tui/layout/README.md)
- [Cached subviews](../widgets.md#subview-composition)

## Source files

- [Layout.fpas](../../../src/Fpas.Ui/Layout.fpas)
- [GUI viewport calculation](../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
- [TUI view preparation](../../../src/Fpas.Ui.Tui/Internal/Hosting/Session.fpas)
