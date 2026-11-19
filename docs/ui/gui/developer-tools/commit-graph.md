# Commit graph

Use a commit graph for history with lanes for branches and merges.
The gallery supplies sample commits with messages and refs.
Available in **GUI**.

![Commit history and branch lanes in the FPAS gallery](images/commit-graph.png)

## Example

```pascal
unit Docs.Ui.DeveloperTools.CommitGraph;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.CommitGraphs as W;

// Displays a repository history already loaded by the application.
// ## Parameters
// - `Commits`: Commits newest first, with parents later in the array.
// - `Selected`: Selected commit index, or minus one.
// ## Returns
// The graph with row selection and activation proposals.
public function View(Commits: array of W.Commit; Selected: int): E.Element<W.Event>;
begin
  return W.CommitGraph<W.Event>('history', Commits, Selected,
    function(Change: W.Event): W.Event => Change, new W.Options(Height := 320.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Parent / MergeParent` | Pass the row index of each parent or minus one when absent. |
| `Hash / Message / Author / Date / Refs` | Pass the hash and message plus optional author, formatted date and ref names. |
| `Width / Height / Enabled` | Fill / 0 / true. Zero uses the native height. |

## Interaction

CUI builds the visible rows as the history scrolls. Up and Down change
selection. Enter or double-click activates a row. Use the proposed index
to resolve the supplied commit.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/CommitGraphs.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)
- [Blame](../more-developer-tools/blame.md)
- [Developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [CommitGraph.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/CommitGraph.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.CommitGraph.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.CommitGraph.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.CommitGraph.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.CommitGraph.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [commit_graph.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/commit_graph.c)
  - [commit_graph.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/commit_graph.h)
