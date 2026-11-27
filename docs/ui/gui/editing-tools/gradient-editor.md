# Gradient editor

Use a gradient editor to set colors and opacity along a ramp. The gallery
shows how flame color changes over its lifetime. Available in **GUI**.

![Flame color ramp in the FPAS gallery](images/gradient-editor.png)

## Example

```pascal
unit Docs.Ui.EditingTools.GradientEditor;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Graphics as G;
uses Fpas.Ui.Widgets.Editors.GradientEditors as W;

// Defines a ramp with an opaque start and transparent end.
// ## Returns
// The initial stops for the model.
public function InitialStops(): array of W.Stop;
begin
  return [new W.Stop(0.0F, new G.Color($FF, $DC, $83)),
    new W.Stop(1.0F, new G.Color($62, $12, $0F, $00))];
end function;

// Edits the current color ramp.
// ## Parameters
// - `Stops`: Model-owned stops sorted by position.
// - `Selected`: Selected stop index, or minus one.
// ## Returns
// The editor with color, position and selection proposals.
public function View(Stops: array of W.Stop; Selected: int): E.Element of W.Event;
begin
  return W.GradientEditor of W.Event('flame-ramp', Stops, Selected,
    function(Change: W.Event): W.Event => Change, new W.Options(Capacity := 12));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Stop.Position / Tint` | Supply a position from zero through one and straight RGBA, including opacity. |
| `Selected` | Pass minus one for no selection. |
| `Width / Height / Capacity` | Fill / 0 / 16. Zero height uses the native ramp height. |

## Interaction

Drag stops or use Left and Right on a focused stop. Double-click a stop or
press Enter to open the color picker. Delete removes a focused stop when
enough stops remain. Store the proposed sorted stops and selected index to
accept edits.
`Changed` identifies an edit to the ramp. `Added` indexes the new ramp,
and `Removed` indexes the previous ramp. Each uses minus one when absent.

## Sample a gradient

Use `Fpas.Ui.Gui.Gradients.Evaluate(Stops, Position)` to sample a color.
Pass finite stop positions from zero through one. Stops can use any order.
CUI interpolates the sRGB channels and alpha between adjacent stops.
Positions outside the ramp use the nearest end color. An empty ramp
returns transparent black, and NaN uses the first stop.
The call works without a window.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_gradient_editor)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Editors/GradientEditors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/EditingTools/View.fpas)
- [Color picker](../inputs/color-picker.md)
- [Editing tools](README.md)

## Source files

- [FPAS ramp sampling](../../../../src/Fpas.Ui.Gui/Gradients.fpas)
- [Gradient data conversion](../../../../src/Fpas.Ui.Gui/Internal/Data/GradientData.fpas)
- [Managed ramp sampling](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiGradient.cs)

- `src/Fpas.Ui.Gui/Internal/Widgets/Editors/`
  - [GradientEditor.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Editors/GradientEditor.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Editors/`
  - [NativeMethods.GradientEditor.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Editors/NativeMethods.GradientEditor.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Editors/`
  - [CuiWindow.GradientEditor.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.GradientEditor.Types.cs)
  - [CuiWindow.GradientEditor.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Editors/CuiWindow.GradientEditor.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/editors/`
  - [gradient_editor.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/gradient_editor.c)
  - [gradient_editor.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/editors/gradient_editor.h)
