# Terminal UI handbook

Build terminal applications with `Fpas.Ui.Tui`. The gallery uses the shared theme and layout rules to place a navigation panel beside a scrollable preview. Its panels share the available width in a 1:2 ratio. Shared examples appear under the same gallery destinations as in the GUI. The cell grid has a separate **Terminal** destination.

![Navigation and progress bars in the terminal gallery](../../../examples/ui/tui/gallery/panels.png)

## First terminal app

Reference `Fpas.Ui.Tui` in the application project. This program opens a single button. Press Enter or Space to exit.

```pascal
program FirstTerminal;

uses Fpas.Ui.Tui.Application as Tui, Fpas.Ui.Mvu.Application as Mvu,
  Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Actions.Buttons as Buttons;

function Update(Model: int; Message: int): Mvu.Transition<int, int>;
begin
  return new Mvu.Transition<int, int>(Model, [Mvu.Exit<int>()]);
end function;

function View(Model: int): E.Element<int>;
begin
  return Buttons.Button<int>('close', 'Close', 1);
end function;

begin
  Tui.Run<int, int>(0, Update, View);
end;

end program;
```

After the [SDK bootstrap](../../build/development.md#build-an-fpas-application), run the full gallery in a terminal.

```sh
dotnet run --project examples/ui/tui/gallery/TuiWidgetGallery.fpasproj
```

## Widget guide

| Category | Widgets and examples |
| --- | --- |
| [Layout](layout/README.md) | Panels, weighted rows and columns, scroll views |
| [Navigation](navigation/README.md) | Sidebar with selectable destinations, tabs, breadcrumbs and pagination |
| [Actions](actions/README.md) | Button, dropdown menu, segmented control and toggle group |
| [Inputs](inputs/README.md) | Text, search and number fields, text area, searchable single and multiple selection |
| [Files](files/README.md) | Open, save and folder selection dialogs |
| [Selection](selection/README.md) | Checkbox, switch and radio group |
| [Overlays](overlays/README.md) | Dialog, sheet and searchable command palette |
| [Data display](data-display/README.md) | Sortable table, selection list, expandable tree and filter bar |
| [Code](developer/README.md) | Scrollback for build and application messages |
| [Disclosure](disclosure/README.md) | Expandable card |
| [Feedback](feedback/README.md) | Inline alert, determinate progress bar and animated spinner |
| [Charts](charts/sparkline.md) | Sparkline for a series of values |
| [App chrome](app-chrome/README.md) | Status bar |
| [Text and display](text-and-display/README.md) | Text, semantic colors and cell grid |

## See also

- [Terminal gallery and key bindings](../../../examples/ui/tui/gallery/README.md)
- [Terminal host and headless sessions](host.md)
- [Shared themes and layout units](../widgets.md#shared-themes)
- [Model–View–Update](../mvu/README.md)
- [Graphical UI](../gui/README.md)
