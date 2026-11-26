# Terminal viewport

Use a `View(Model, Layout.Viewport)` with `Host.Run` or `Headless.Start` to
select a layout by width and height class. The TUI host derives both classes
from the terminal's column and row counts. It supplies `LogicalWidth` as
eight units per column and `LogicalHeight` as 16 units per row.

Use `TerminalExtensions.Views.Viewport` when a subtree needs exact counts.
Give the subtree a stable ID. The builder receives a `Terminal.Viewport`
with `Columns` and `Rows` for the whole terminal. The host resolves and
validates the subtree during view preparation.

A cell grid reports its own allocation through `Resized`.
[Cell grid](../text-and-display/cell-grid.md)

## Example

```pascal
unit Docs.Tui.Layout.ExactViewport;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.TerminalExtensions.Views as Views;
uses Fpas.Ui.Widgets.Developer.Terminal as Terminal;

function Build(Size: Terminal.Viewport): E.Element of string;
begin
  if Size.Columns < 50 then
    return E.Column of string(E.Text of string('Summary'), E.Text of string('Details'));
  end if;

  return E.Row of string(E.Text of string('Summary'), E.Text of string('Details'));
end function;

// Builds a view from the exact terminal width.
// ## Returns
// A terminal subtree that changes its arrangement at 50 columns.
public function View(): E.Element of string;
begin
  return Views.Viewport of string('summary-layout', Build);
end function;

end unit;
```

## Resize and screen composition

An interactive resize updates the viewport before the host prepares the
next view. In a headless session, send `Application.Input.Resize(Columns,
Rows)` and call `Step`. Use 1 to 500 columns and 1 to 300 rows. Inspect the
new dimensions and visible control bounds in `Snapshot`.

Use `Views.Screen` to compose base content, overlay elements in paint order
and base-scope shortcuts. Put the viewport-dependent subtree in the base
content when the terminal size changes its structure.

## See also

- [Shared responsive layout and size classes](../../layout/responsive.md)
- [Interactive and headless terminal hosts](../host.md#application-lifecycle)
- [Terminal overlays](../overlays/README.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Views.fpas](../../../../src/Fpas.Ui/TerminalExtensions/Views.fpas)
- [Terminal.fpas](../../../../src/Fpas.Ui/Widgets/Developer/Terminal.fpas)
- [Application.fpas](../../../../src/Fpas.Ui.Tui/Application.fpas)
- [Host.fpas](../../../../src/Fpas.Ui.Tui/Host.fpas)
- [Headless.fpas](../../../../src/Fpas.Ui.Tui/Headless.fpas)
- [Session.fpas](../../../../src/Fpas.Ui.Tui/Internal/Hosting/Session.fpas)
