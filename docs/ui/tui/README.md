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
| [Navigation](navigation/README.md) | Sidebar with selectable destinations |
| [Actions](actions/README.md) | Button |
| [Inputs](inputs/README.md) | Single-line text field, select with optional search |
| [Selection](selection/README.md) | Checkbox, switch and radio group |
| [Overlays](overlays/README.md) | Dialog and sheet |
| [Data display](data-display/README.md) | Table with sortable headings and selectable rows |
| [Feedback](feedback/README.md) | Determinate progress bar |
| [App chrome](app-chrome/README.md) | Status bar |
| [Text and display](text-and-display/README.md) | Text, semantic colors and cell grid |

## See also

- [Terminal gallery and key bindings](../../../examples/ui/tui/gallery/README.md)
- [Terminal host and headless sessions](host.md)
- [Shared themes and layout units](../widgets.md#shared-themes)
- [Model–View–Update](../mvu/README.md)
- [Graphical UI](../gui/README.md)
