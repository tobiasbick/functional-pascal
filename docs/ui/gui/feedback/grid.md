# Grid

Use a grid for equal-width columns that can wrap as the window narrows. The gallery places Activity and Goals in the same row. Available in **GUI**.

![Grid in the FPAS gallery](images/grid.png)

## Example

```pascal
unit Docs.Ui.Feedback.Grid;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Containers.Grids as W;

// Builds the grid example.
// ## Parameters
// - `Activity`: Activity cell.
// - `Goals`: Goals cell.
// ## Returns
// The widget for the application view.
public function View(Activity: E.Element<string>; Goals: E.Element<string>): E.Element<string>;
begin
  return W.Grid<string>(
    new W.Options(Columns := 2, MinimumColumnWidth := 260.0F,
      Gap := L.Spacing.Fixed(L.Measure.Logical(16))),
    Activity, Goals);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Columns` | 1. Sets the maximum column count. |
| `MinimumColumnWidth` | 0 keeps the column count fixed. A positive logical width lets CUI reduce the number of columns. |
| `Gap / Padding` | None / None. |

## Interaction

CUI sizes each row to its tallest cell and places children in the order supplied. Use the enclosing layout to constrain the available width. For spacing inside a child row or column, use `Layout.Flow.Justify`. The Goals card uses Around to distribute the two rings. Overview uses Grid for both its metric tiles and its workspace and team cards.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/Grids.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- [Card](card.md)
- [Overview](../overview/README.md)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Containers/`
  - [Grid.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/Grid.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Containers/`
  - [NativeMethods.Grid.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Containers/NativeMethods.Grid.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Containers/`
  - [CuiWindow.Grid.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Containers/CuiWindow.Grid.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/containers/`
  - [grid.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/grid.c)
  - [grid.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/grid.h)
