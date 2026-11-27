# Problems panel

Use a problems panel for compiler or lint diagnostics. The gallery
groups entries by file and lets users filter by severity or text.
Available in **GUI**.

![Grouped diagnostics in the FPAS gallery](images/problems-panel.png)

## Example

```pascal
unit Docs.Ui.DeveloperTools.ProblemsPanel;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.ProblemsPanels as W;

// Displays the latest diagnostics.
// ## Parameters
// - `Diagnostics`: Diagnostics in the application model.
// ## Returns
// The panel with selection and source activations as messages.
public function View(Diagnostics: array of W.Diagnostic): E.Element of W.Event;
begin
  return W.Problems of W.Event('diagnostics', Diagnostics,
    function(Change: W.Event): W.Event => Change, new W.Options(Height := 300.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Severity` | 0 error, 1 warning, 2 info, 3 hint. |
| `File / Line / Column` | Use null for an entry without a file. Lines and columns are one-based. Zero omits a location component. |
| `Source / Code` | Optional diagnostic origin and identifier. |
| `Width / Height` | Fill / 0. Zero height uses 300 logical units. Negative height fills the parent. |
| `HideToolbar / Enabled` | false / true. `HideToolbar` hides severity filters and search. |

## Interaction

Up and Down move selection. Home and End go to the first and last rows.
Use Left and Right to fold or unfold groups. Enter toggles a focused group.
Enter or double-click on a diagnostic activates it. A focus request for the
panel targets the list.

Resolve `Activated` against the input array to open the source location.
The widget also reports selection and filter changes with `Activated` set
to -1. `Shown` counts diagnostics that match the filters, including folded
groups. `Selected` is -1 when a group is selected.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_problems)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/ProblemsPanels.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)
- [Developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [Problems.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/Problems.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.Problems.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.Problems.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.Problems.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Problems.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [problems.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/problems.c)
  - [problems.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/problems.h)
