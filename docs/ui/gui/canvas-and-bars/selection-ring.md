# Selection ring

Use a selection ring to identify another person's selection. The gallery
places Paul's ring on the Generate node.
Available in **GUI**.

![Paul's selection ring around the Generate node in the FPAS gallery](images/selection-ring.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.SelectionRing;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Canvas.SelectionRings as W;

// Marks a selected object with its owner's name.
// ## Parameters
// - `Child`: Object that receives the ring.
// - `Owner`: Collaborator label.
// ## Returns
// The child with the native selection ring.
public function View(Child: E.Element<string>; Owner: string): E.Element<string>;
begin
  return W.SelectionRing<string>('remote-selection', Child,
    new W.Options(Tone := S.Tone.Warning, Name := Owner));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Tone / Color / Name` | Neutral / null / null. A color overrides the tone. Supply Name to label the ring. |
| `Radius` | 0. Zero follows the child's corner radius. |

## Interaction

Wrap the selected child, or set the node's `Ring` options if the child
is a node inside a node editor. Draw a new ring when the model changes
the selected object.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Canvas/SelectionRings.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Node editor](node-editor.md)
- [Canvas and bars](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [SelectionRing.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/SelectionRing.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.SelectionRing.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.SelectionRing.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiWindow.SelectionRing.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.SelectionRing.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [selection_ring.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/selection_ring.c)
  - [selection_ring.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/selection_ring.h)
