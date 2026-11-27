# Source list

The gallery shows source titles and domains below the answer.
Available in **GUI**.

![Source cards in the FPAS gallery](images/source-list.png)

## Example

```pascal
unit Docs.Ui.Ai.SourceList;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.SourceLists as W;
uses Fpas.Ui.Widgets.Ai.Sources as S;

// Shows sources collected by the application.
// ## Parameters
// - `Documents`: Sources in display order.
// ## Returns
// The list with source indices as activation messages.
public function View(Documents: array of S.Item): E.Element of int;
begin
  return W.SourceList of int('sources', Documents, function(Index: int): int => Index);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Documents` | Pass sources in display order. The list numbers the cards from one. |
| `Source.Url` | Null. A supplied URL sets the displayed domain and the card tooltip. |
| `Source.Title` | Null uses the domain. An empty string gives a blank title. |
| `Source.Snippet` | Null. Citations use this text in previews. |
| `Changed / Enabled` | Pass a mapper to handle source activation, or null to omit the message. `Enabled` defaults to true. |

## Interaction

Scroll sideways to see more source cards. Use Tab to move focus. The
list scrolls to show a focused card if it is outside the viewport.
Click a card or press Enter or Space when it has focus. Resolve the
zero-based index against the supplied documents and open the source in
the application.

The tooltip displays up to 255 UTF-8 bytes of the URL and ends at a
character boundary.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_source_list)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/SourceLists.fpas)
- [Source type](../../../../src/Fpas.Ui/Widgets/Ai/Sources.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [Citation](citation.md)
- [AI](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [SourceList.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/SourceList.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.SourceList.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.SourceList.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.SourceList.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.SourceList.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [source_list.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/source_list.c)
  - [source_list.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/source_list.h)
