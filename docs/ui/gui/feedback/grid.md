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
public function View(Activity: E.Element of string; Goals: E.Element of string): E.Element of string;
begin
  return W.Grid of string(
    new W.Options(Columns := 0, MinimumColumnWidth := 260.0F,
      Flow := new L.Flow(Gap := L.Spacing.Fixed(L.Measure.Logical(16)))),
    Activity, Goals);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Columns` | Defaults to 1. A positive value sets the fixed count or responsive cap. With a positive minimum width, zero lets CUI fit as many columns as the width and child count allow. Zero with no minimum uses one column. |
| `MinimumColumnWidth` | 0 keeps the column count fixed. A positive logical width makes the count adapt to the available width. |
| `Flow` | Optional `Layout.Flow` for spacing, edge insets, direction, clipping, appearance, native scrolling, focus scope and stable identity. Its gap and padding take precedence when supplied. |
| `Gap / Padding` | Both default to None and apply when `Flow` is omitted. |

## Interaction

CUI sizes each row to its tallest cell and places children in the order supplied.
Use `Elements.Size` to set a width, height or bounds on the grid. The grid
positions children at cell origins. Set `Align` and `Justify` on a row or
column inside a cell to arrange its content. The Goals card uses Around to
distribute the two rings. Overview uses Grid for both its metric tiles and
its workspace and team cards.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/Grids.fpas)
- [Common box options](../layout/column.md)
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
- [FPAS layout contract](../../../../tests/Fixtures/gui-host/LayoutContract.fpas)
- [Native layout comparison](../../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
