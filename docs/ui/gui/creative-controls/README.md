# Creative controls

The gallery edits image settings and shows native action wells. Values and
selections live in the application model. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Status tag](status-tag.md) | Show a named workflow state. |
| [Ratio slider](ratio-slider.md) | Set the split between two named parts. |
| [Gradient slider](gradient-slider.md) | Pick a value along a color ramp. |
| [Step slider](step-slider.md) | Choose an integer stop with an optional action. |
| [Palette strip](palette-strip.md) | Lock swatches and shuffle the remaining colors. |
| [Aspect ratio picker](aspect-ratio-picker.md) | Pick proportions from previews of the shapes. |
| [Option field with action](option-action.md) | Choose an option and act on it. |
| [Seed field](seed-field.md) | Roll a seed and optionally lock it. |

## See also

- [UI handbook](../README.md)
- [Canvas and bars](../canvas-and-bars/README.md)
- [Creative controls in the gallery](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Creative/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Creative/Update.fpas)
- [creative.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/creative.c)
