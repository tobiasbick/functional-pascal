# Editing tools

The gallery inspects a crate material, profiles two frames and edits a shot.
The model holds editable values, selections and view ranges. These widgets
require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Property inspector](property-inspector.md) | Edit grouped fields with controls chosen by value type. |
| [Flame graph](flame-graph.md) | Inspect timed spans by stack depth. |
| [Curve editor](curve-editor.md) | Edit key values, handles and interpolation. |
| [Gradient editor](gradient-editor.md) | Edit positions and colors along a ramp. |
| [Timeline editor](timeline-editor.md) | Edit keyframes and clips, scrub time and request playback. |

A timeline key holds a curve key. The gallery lets you edit the selected
track's keys in a separate curve editor while sharing its playhead.

## See also

- [UI handbook](../README.md)
- [Code](../code/README.md)
- [Editing tools in the gallery](../../../../examples/ui/gui/gallery/Pages/EditingTools/View.fpas)

## Source files

- [Samples.fpas](../../../../examples/ui/gui/gallery/Pages/EditingTools/Samples.fpas)
- [State.fpas](../../../../examples/ui/gui/gallery/Pages/EditingTools/State.fpas)
- [editing_tools.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/editing_tools.c)
- [EditingToolsContract.fpas](../../../../tests/Fixtures/gui-host/EditingToolsContract.fpas)
