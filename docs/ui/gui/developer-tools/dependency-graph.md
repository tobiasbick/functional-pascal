# Dependency graph

Use a dependency graph to see how components are connected. The gallery
lays out build dependencies in layers.
Available in **GUI**.

![Layered dependency graph in the FPAS gallery](images/dependency-graph.png)

## Example

```pascal
unit Docs.Ui.DeveloperTools.DependencyGraph;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.DependencyGraphs as W;

// Shows a directed graph.
// ## Parameters
// - `Vertices`: Nodes in model order.
// - `Edges`: Directed edges between zero-based node indices.
// - `Selected`: Selected node index, or minus one.
// ## Returns
// The graph with selection and activation as application messages.
public function View(Vertices: array of W.Vertex; Edges: array of W.Edge; Selected: int): E.Element of W.Event;
begin
  return W.DependencyGraph of W.Event('dependencies', Vertices, Edges, Selected,
    function(Change: W.Event): W.Event => Change, new W.Options(Height := 340.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Vertex.Label / Detail / Tone` | Supply the label, a nullable detail and a `Fpas.Ui.Styles.Tone` ordinal. |
| `Edge.From / To` | Pass zero-based node indices. CUI computes layers and breaks cycles in the layout. |
| `TopDown / Width / Height / Enabled` | false / Fill / 0 / true. TopDown stacks the layers vertically. Height zero uses 360 logical units. |
| `Selection` | Pass the selected node, minus one for none, or null to let CUI retain the selection. |
| `Hovered` | null. Set minus one to start tracking hover. Store the last `Event.Hovered` value to receive changes only. |

## Interaction

Hover a node to highlight its dependencies and dependants. Drag to pan, use
the wheel to zoom, or hold Shift while scrolling to pan horizontally.
Press F to fit the graph and use plus or minus to adjust zoom.

Click a node to select it. Arrow keys move along edges or across a layer.
Enter or double-click activates the selected node. Escape clears selection.
Request focus with the graph's stable ID.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_graph_view)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/DependencyGraphs.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)
- [Developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [DependencyGraph.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/DependencyGraph.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.DependencyGraph.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.DependencyGraph.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.DependencyGraph.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.DependencyGraph.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [dependency_graph.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/dependency_graph.c)
  - [dependency_graph.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/dependency_graph.h)
- [Native graph tests](../../../../src/Fpas.Ui.Gui.Cui/native/tests/developer_graphs.c)
