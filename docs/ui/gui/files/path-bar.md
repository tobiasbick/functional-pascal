# Path bar

Use a path bar for navigation through a folder hierarchy. CUI draws the
segments, offers child folders in menus and accepts a typed path.
Available in **GUI**.

![Standalone path bar in the FPAS gallery](images/path-bar.png)

## Example

```pascal
unit Docs.Ui.Files.PathBar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Navigation.PathBars as W;

// Shows a path with alternate folders under project.
// ## Returns
// The path bar with navigation proposals as messages.
public function View(): E.Element<W.Event>;
begin
  const Current: W.Value := new W.Value(['project', 'src'], 'project/src',
    [['assets', 'src']]);
  return W.PathBar<W.Event>('location', Current,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Value.Segments` | Supply segments in root-to-leaf order. |
| `Value.Text / Children` | Empty text joins the segments. Children defaults to empty. Each supplied child list belongs to the segment at the same index. |
| `Icon / Placeholder` | Both empty. |
| `ControlSize / Width` | Medium / Fill. |
| `Editable / Enabled` | Both true. |

## Interaction

Use `Event.Segment` to navigate to a segment, `MenuSegment` and `Child` to
resolve a selected menu item, or `Text` to handle a submitted path. Unused
indices are minus one. Resolve or validate the destination in the update
and pass the resulting segments and menus back to the view.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/PathBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)
- [File browser](file-browser.md)
- [Files](README.md)
