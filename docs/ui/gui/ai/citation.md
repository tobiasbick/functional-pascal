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
// - `Source`: Optional source URL, title and snippet.
// ## Returns
// The citation chip.
public function View(Number: int; Source: S.Item): E.Element of string;
begin
  return W.Citation of string('reference', Number, Source,
    function(): string => Source.Url ?? '');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Number` | Pass a positive number. The chip shows this number if the source has no domain. |
| `Source` | Pass an optional source. Null omits the preview. |
| `Source.Url / Title / Snippet` | Each defaults to null. The URL supplies the domain. The title and snippet supply text for the preview. |
| `Activate / Enabled` | Pass a callback to handle activation, or null to omit the message. `Enabled` defaults to true. |

## Interaction

Hover a chip with a source to view its preview. To activate an enabled
chip, click it or press Enter or Space when it has focus. Handle the
activation message in the application to open the source.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_citation)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/Citations.fpas)
- [Source type](../../../../src/Fpas.Ui/Widgets/Ai/Sources.fpas)
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
