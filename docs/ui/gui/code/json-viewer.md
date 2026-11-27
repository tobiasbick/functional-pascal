# JSON viewer

Use a JSON viewer to inspect a document. The gallery shows an expandable
object and a separate example with a parse error. Available in **GUI**.

![Expandable JSON document in the FPAS gallery](images/json-viewer.png)

## Example

```pascal
unit Docs.Ui.Code.JsonViewer;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.JsonViews as W;

// Displays the current JSON document.
// ## Parameters
// - `Document`: Literal JSON source.
// ## Returns
// The native JSON viewer for the application view.
public function View(Document: string): E.Element of string;
begin
  return W.JsonView of string('response', Document, new W.Options(ExpandDepth := 2));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Json` | Pass source text. CUI parses the document and displays errors in place. |
| `ExpandDepth` | Zero uses CUI's two-level default. One expands only the root container. |
| `MaxHeight` | Zero leaves the height to the enclosing layout. A positive value caps the viewport in logical units. |

## Interaction

Click an object or array row to fold or unfold it. Folded objects show the
number of keys. Folded arrays show the number of items. CUI applies the
initial depth once and preserves folding by position for the mounted
widget. Use a new ID to start with the initial depth for another document.

Scroll vertically when the content exceeds the viewport height. If the
document shrinks while scrolled, the host requests a frame to clamp the
scroll position. [CUI-065](../../../future/cui-upstream-bugs.md#cui-065)

The `json-view` snapshot reports `Value` as `True` for a parsed
document or `False` for an error.

![JSON parse error in the FPAS gallery](images/json-viewer-error.png)

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_json_view)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/JsonViews.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Code/View.fpas)
- [Code](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [JsonView.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/JsonView.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.JsonView.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.JsonView.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.JsonView.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.JsonView.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [json_view.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/json_view.c)
  - [json_view.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/json_view.h)
