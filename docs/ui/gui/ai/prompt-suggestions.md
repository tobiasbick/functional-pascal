# Prompt suggestions

The gallery has four suggestions beside its conversation. Choosing one
adds its title to the draft.
Available in **GUI**.

![Prompt suggestion cards in the FPAS gallery](images/prompt-suggestions.png)

## Example

```pascal
unit Docs.Ui.Ai.PromptSuggestions;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.PromptSuggestions as W;

// Offers example starting prompts.
// ## Parameters
// - `Items`: Suggestion cards in display order.
// ## Returns
// The cards with the chosen index as an application message.
public function View(Items: array of W.Item): E.Element of int;
begin
  return W.PromptSuggestions of int('prompts', Items, function(Index: int): int => Index,
    new W.Options(Columns := 1));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Item.Title / Detail / Icon` | Supply a title. Detail and Icon default to empty. |
| `Columns / Enabled` | 0 / true. Zero uses the native column count. |

## Interaction

Look up the selected index and apply its text to the draft. Sending
a request is a separate application action.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_prompt_suggestions)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/PromptSuggestions.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [Composer](../voice/composer.md)
- [AI](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [PromptSuggestions.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/PromptSuggestions.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.PromptSuggestions.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.PromptSuggestions.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.PromptSuggestions.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.PromptSuggestions.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [prompt_suggestions.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/prompt_suggestions.c)
  - [prompt_suggestions.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/prompt_suggestions.h)
