# HTTP request bar

Use a request bar for an HTTP draft. The gallery keeps its URL in sync
with a separate query-parameter editor.
Available in **GUI**.

![HTTP method, URL and parameters in the FPAS gallery](images/http-request-bar.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.HttpRequestBar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.RequestBars as W;

// Shows the current request draft and loading state.
// ## Parameters
// - `Method`: Current native HTTP method index.
// - `Url`: Current draft URL.
// - `Loading`: Whether an application request is in progress.
// ## Returns
// The request bar with edits and send or cancel intents.
public function View(Method: int; Url: string; Loading: bool): E.Element<W.Event>;
begin
  return W.RequestBar<W.Event>('request', Method, Url,
    function(Change: W.Event): W.Event => Change, new W.Options(Loading := Loading));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Method` | 0 GET, 1 POST, 2 PUT, 3 PATCH, 4 DELETE, 5 HEAD, 6 OPTIONS. |
| `Capacity / Placeholder` | 255 / empty. Capacity limits the UTF-8 URL edit buffer. |
| `Loading / Width / Enabled` | false / Fill / true. Loading replaces Send with Cancel. |

## Interaction

Store the method and URL from the proposal. Send or Enter in the URL
requests submission. During loading, the button requests cancellation.
Your application handles both intents. The gallery works with local
sample data.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/RequestBars.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [HTTP response status](http-response-status.md)
- [Key-value editor](../developer-tools/key-value-editor.md)
- [Gallery update function](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/Update.fpas)
- [More developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [RequestBar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/RequestBar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.RequestBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.RequestBar.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.RequestBar.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.RequestBar.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [request_bar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/request_bar.c)
  - [request_bar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/request_bar.h)
