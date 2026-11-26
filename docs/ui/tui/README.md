# Terminal UI handbook

Build terminal applications with `Fpas.Ui.Tui`. The gallery uses the shared
theme and layout rules to place a narrow navigation panel beside a scrollable
preview. The [fractal explorer](../../../examples/ui/tui/fractals/README.md)
demonstrates a keyboard-controlled cell grid.

![Keyboard-driven gallery at 80 × 25 showing navigation and progress](../../../examples/ui/tui/gallery/panels.png)

## Choosing terminal widgets

Design for an SSH terminal with 80 columns and 25 rows and keyboard input.
Start with a task that is useful in that environment. A CUI example can
supply sample data and intent. Choose the layout and interaction that make
the task practical in a terminal.

Check the primary workflow from entry to completion at 80 × 25 using only
the keyboard. The user needs to find the action, follow focus when the view
scrolls, read the information needed to make a choice, and see the result.
Display widgets need a readable representation at this size. Preserve the
essential information with text or terminal glyphs.

Keep desktop workflows in the GUI when they depend on precise pointer
positioning, drag gestures or a large visual canvas. Add a terminal control
when it solves a useful terminal task and meets the keyboard and space
requirements.

## First terminal app

Reference `Fpas.Ui.Tui` in the application project. This program opens a single button. Press Enter or Space to exit.

```pascal
program FirstTerminal;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Mvu.Application as Mvu;
uses Fpas.Ui.Tui.Host as Tui;
uses Fpas.Ui.Widgets.Actions.Buttons as Buttons;

function Update(Model: int; Message: int): Mvu.Transition of (int, int);
begin
  return new Mvu.Transition of (int, int)(Model, [Mvu.Exit of int()]);
end function;

function View(Model: int): E.Element of int;
begin
  return Buttons.Button of int('close', 'Close', 1);
end function;

begin
  Tui.Run of (int, int)(0, Update, View);
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
| [Actions](actions/README.md) | Button, dropdown menu and segmented control |
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
