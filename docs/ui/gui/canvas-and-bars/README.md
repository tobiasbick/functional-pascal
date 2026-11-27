# Canvas and bars

The gallery combines a node canvas, collaborator overlays and action bars.
The application owns the viewport, nodes, selections and action state.
These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Toolbar](toolbar.md) | Group actions, modes and cycling choices. |
| [Format bar](format-bar.md) | Apply text styles and actions to a selection. |
| [Node editor](node-editor.md) | Edit a graph of connected nodes and pins. |
| [Canvas controls](canvas-controls.md) | Add a zoom rail with fit and toggles. |
| [Live cursors](live-cursors.md) | Overlay collaborators' positions on a canvas. |
| [Presence pill](presence-pill.md) | Show who is present and who is speaking. |
| [Selection ring](selection-ring.md) | Outline a selection and show its owner's name. |
| [Result card](result-card.md) | Show a picture with a caption and result status. |
| [Selection bar](selection-bar.md) | Offer actions for a multi-item selection. |
| [Custom canvas painting](painting.md) | Draw with the final canvas size and receive pointer or wheel events. |

## See also

- [UI handbook](../README.md)
- [Creative controls](../creative-controls/README.md)
- [Canvas and bars in the gallery](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Comparison with the CUI gallery](../../../future/cui-compare/README.md#canvas-and-bars-evidence)

## Source files

- `examples/ui/gui/gallery/Pages/Canvas/`
  - [Picture.fpas](../../../../examples/ui/gui/gallery/Pages/Canvas/Picture.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Canvas/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Canvas/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeCanvasData.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeCanvasData.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [canvas_bars.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/canvas_bars.c)
- `tests/Fixtures/gui-host/`
  - [CanvasBarsContract.fpas](../../../../tests/Fixtures/gui-host/CanvasBarsContract.fpas)
