# Cell grid

Draw a character and foreground/background colors per cell. The fractal
explorer uses a cell grid for the plot and binds keyboard shortcuts to pan,
zoom and adjust the iteration limit.

## Example

```pascal
unit Docs.Tui.Widgets.CellGrid;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.TerminalExtensions.CellGrids as W;

// Builds the cell grid example.
// ## Parameters
// - `Image`: Current pixels, or null while computing them.
// ## Returns
// The resulting view.
public function View(Image: W.Image?): E.Element<W.Event>;
begin
  return W.CellGrid<W.Event>('drawing', Image, function(Input: W.Event): W.Event => Input);
end function;

end unit;
```

## Interaction

On `Resized`, use the reported columns and rows to create the image with `Image.Create`. A click reports zero-based local coordinates and the mouse button. The wheel reports a signed distance. Each image slot occupies one column.

## See also

- [Text and display](README.md)
- [Terminal host reference](../host.md#terminal-extensions)
- [Fractal explorer](../../../../examples/ui/tui/fractals/README.md)

## Source files

- [CellGrids.fpas](../../../../src/Fpas.Ui/TerminalExtensions/CellGrids.fpas)
- [CellGrid.fpas](../../../../tests/Fixtures/tui/CellGrid.fpas)
- [RenderingContract.fpas](../../../../tests/Fixtures/tui/RenderingContract.fpas)
