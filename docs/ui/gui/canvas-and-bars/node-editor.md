# Node editor

Use a node editor for a visual graph with composed node bodies.
The gallery connects prompt nodes to an image generation node.
Available in **GUI**.

![Nodes, pins and links in the FPAS gallery](images/node-editor.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.NodeEditor;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Editors.NodeEditors as W;

// Shows a model-owned graph.
// ## Parameters
// - `Viewport`: Pan position and zoom.
// - `Nodes`: Nodes with unique IDs, positions, pins and optional bodies.
// - `Links`: Connections from output to input pin IDs.
// ## Returns
// The node editor with native input proposals.
public function View(Viewport: W.View; Nodes: array of W.Node<W.Event>; Links: array of W.Link): E.Element<W.Event>;
begin
  return W.NodeEditor<W.Event>('graph', Viewport, Nodes, Links,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Pin.Id / Output / Kind` | Use a unique pin ID, a direction and a connection type. Kind zero accepts any type. |
| `Node.Body / Width / Ring` | null / 0 / null. Zero uses native width. Ring can show a collaborator's selection. |
| `Width / Height / MinimumZoom / MaximumZoom` | Fill / 440 logical points / 0 / 0. Zero zoom bounds use native defaults. |
| `HideGrid / Minimap / Enabled` | false / false / true. |

## Interaction

Drag nodes or connections. CUI proposes pan and zoom, node movements,
new links and removal actions. Decide which changes to accept and render
the resulting graph. A zoom of one is 100 percent.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Editors/NodeEditors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Canvas controls](canvas-controls.md)
- [Selection ring](selection-ring.md)
- [Canvas and bars](README.md)
