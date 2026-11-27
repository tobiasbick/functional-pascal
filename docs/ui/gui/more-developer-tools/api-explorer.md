# API explorer

Use an API explorer to browse methods and paths grouped by tag. The
gallery shows Users and Orders, including a deprecated endpoint.
Available in **GUI**.

![Endpoint groups in the FPAS gallery](images/api-explorer.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.ApiExplorer;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.ApiExplorers as W;

// Shows the API catalog loaded by the application.
// ## Parameters
// - `Endpoints`: Endpoints and their parameter metadata.
// - `Selected`: Selected endpoint index, or minus one.
// ## Returns
// The explorer with selection and Try it requests as messages.
public function View(Endpoints: array of W.Endpoint; Selected: int): E.Element of W.Event;
begin
  return W.ApiExplorer of W.Event('endpoints', Endpoints, Selected,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Endpoint.Method / Path / Tag` | Supply method and path. A null tag groups the endpoint under Other. |
| `Deprecated / Parameters` | Deprecated draws a badge. Parameters supply names, locations, types, required flags and optional descriptions. |
| `Selection` | Use an endpoint index, minus one for no selection, or null for native selection. |
| `Width / Height / Enabled` | Fill / 0 / true. Zero height fits the content. A negative height fills the parent. |

## Interaction

Filter by path or summary. Click a tag to fold its group. Use Up and Down
to move between visible endpoints. Click an endpoint or press Enter or
Space to show its parameters.

Store `Event.Selection` if the model owns the selection. `Event.TryIt`
contains the requested endpoint index or minus one. Use the index to
populate a request draft in your application.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_api_explorer)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/ApiExplorers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [HTTP request bar](http-request-bar.md)
- [More developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [ApiExplorer.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/ApiExplorer.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.ApiExplorer.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.ApiExplorer.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.ApiExplorer.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.ApiExplorer.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [api_explorer.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/api_explorer.c)
  - [api_explorer.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/api_explorer.h)
- [Native regression tests](../../../../src/Fpas.Ui.Gui.Cui/native/tests/developer_editors.c)
