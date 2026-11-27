# Context meter

The gallery summarizes context usage from system text, files and messages.
Available in **GUI**.

![Context usage indicator in the FPAS gallery](images/context-meter.png)

## Example

```pascal
unit Docs.Ui.Ai.ContextMeter;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.ContextMeters as W;

// Shows the current context usage.
// ## Parameters
// - `Used`: Amount used.
// - `Limit`: Available context capacity.
// - `Parts`: Labeled amounts in the usage breakdown.
// ## Returns
// The native context indicator.
public function View(Used: float; Limit: float; Parts: array of W.Part): E.Element of string;
begin
  return W.ContextMeter of string('context', Used, Limit, Parts, new W.Options(Unit := 'tokens'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Used / Limit` | Use consistent units for current usage and capacity. Pass a positive capacity. |
| `Parts` | Pass labels and amounts for the breakdown, or an empty array to omit the breakdown bar. |
| `Part.Color` | Automatic uses the theme palette. Supply a color for a specific part. |
| `Unit` | Null uses the native `tokens` label. An empty string omits the unit label. |

## Interaction

The ring uses the warning color at 80 percent and the danger color at
95 percent. Hover to see the totals and breakdown. The breakdown bar
adds a free segment for any capacity left after the supplied parts.
Pass updated totals as the conversation adds or removes context.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_context_meter)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/ContextMeters.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [AI](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [ContextMeter.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/ContextMeter.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.ContextMeter.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.ContextMeter.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.ContextMeter.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.ContextMeter.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [context_meter.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/context_meter.c)
  - [context_meter.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/context_meter.h)
