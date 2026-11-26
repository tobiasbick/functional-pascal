# Blame

Use Blame to show authors and revisions next to source lines. The
gallery highlights all lines associated with a gutter entry.
Available in **GUI**.

![Source lines and revision gutter in the FPAS gallery](images/blame.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.Blame;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.Blames as W;

// Displays source with the revision that last changed each line.
// ## Parameters
// - `Lines`: Source lines in display order.
// - `LineCommits`: One revision index per source line.
// - `Revisions`: Hashes, authors, summaries and timestamps.
// ## Returns
// The blame view with clicked revisions as messages.
public function View(Lines: array of string; LineCommits: array of int; Revisions: array of W.Revision): E.Element<W.Event>;
begin
  return W.Blame<W.Event>('source-history', Lines, LineCommits, Revisions,
    function(Change: W.Event): W.Event => Change,
    new W.Options(Language := 'c', Height := 320.0F, FirstLine := 120));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `LineCommits` | Supply one index into Revisions for each line. Use minus one for unattributed lines. |
| `Revision.Time / Now` | Both use Unix seconds. Zero Now uses the current clock for relative ages. |
| `Language / FirstLine / Height` | Empty / 0 / 0. Zero starts numbering at one or uses the native height. |
| `Width / Enabled` | Fill / true. |

## Interaction

Hover the gutter for a summary and to highlight the revision's lines.
Click to receive `Event.Clicked`, an index into the supplied revisions.
Use it to open the commit in your application's history view.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/Blames.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [Commit graph](../developer-tools/commit-graph.md)
- [More developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [Blame.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/Blame.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.Blame.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.Blame.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.Blame.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Blame.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [blame.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/blame.c)
  - [blame.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/blame.h)
