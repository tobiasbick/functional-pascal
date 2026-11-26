# Citation

The gallery puts three citations beneath the answer. When a source is
supplied, the native chip can show its domain and a preview.
Available in **GUI**.

![Inline citation chips in the FPAS gallery](images/citation.png)

## Example

```pascal
unit Docs.Ui.Ai.Citation;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.Citations as W;
uses Fpas.Ui.Widgets.Ai.Sources as S;

// Shows a source reference with an activation message.
// ## Parameters
// - `Number`: Citation number used when no source domain is available.
// - `Source`: Source URL, title and optional snippet.
// ## Returns
// The citation chip.
public function View(Number: int; Source: S.Item): E.Element of string;
begin
  return W.Citation of string('reference', Number, Source,
    function(): string => Source.Url);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Number / Source` | Pass the number and an optional source. A null source gives a numbered chip. |
| `Activate / Enabled` | Pass a callback to handle activation, or null to omit the message. Enabled defaults to true. |

## Interaction

Hover the chip to view the source preview. When a callback is supplied,
handle activation in the application and open the intended source.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_citation)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/Citations.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [Source list](source-list.md)
- [AI](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [Citation.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/Citation.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.Citation.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.Citation.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.Citation.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.Citation.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [citation.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/citation.c)
  - [citation.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/citation.h)
