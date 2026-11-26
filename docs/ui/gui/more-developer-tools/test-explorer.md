# Test explorer

Use a test explorer to show suites, results and failure details.
The gallery includes passing, failing and skipped specimens.
Available in **GUI**.

![Test suites and run results in the FPAS gallery](images/test-explorer.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.TestExplorer;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.TestExplorers as W;

// Displays the current test run.
// ## Parameters
// - `Items`: Tests and suites in tree order.
// ## Returns
// The explorer with test and source-navigation requests as messages.
public function View(Items: array of W.Item): E.Element of W.Event;
begin
  return W.TestExplorer of W.Event('test-results', Items,
    function(Change: W.Event): W.Event => Change, new W.Options(Height := 500.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Item.Depth` | Depth zero starts a top-level item. List the children after their suite. |
| `Item.Status / Duration / Message` | Use the pipeline status codes. Pass seconds or a negative duration, and an optional failure message. |
| `Width / Height / Enabled` | Fill / 0 / true. Zero uses the native height. |

## Interaction

Expand suites, filter by name or status, and press Enter to activate the
selected item. `Run` identifies the row whose run button was used, or
minus one. `RunAll` requests the full run. Handle execution and results
in the application. Reported totals count tests.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_test_explorer)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/TestExplorers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [Pipeline](../developer-tools/pipeline.md)
- [More developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [TestExplorer.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/TestExplorer.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.TestExplorer.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.TestExplorer.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.TestExplorer.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.TestExplorer.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [test_explorer.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/test_explorer.c)
  - [test_explorer.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/test_explorer.h)
