# Live cursors

Use live cursors for a shared canvas. The gallery shows named pointers
and a short speech bubble.
Available in **GUI**.

![Collaborator pointers on the node canvas in the FPAS gallery](images/node-editor.png)

## Example

```pascal
unit Docs.Ui.CanvasAndBars.LiveCursors;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Canvas.LiveCursors as W;
uses Fpas.Ui.Widgets.Editors.NodeEditors as N;

// Overlays collaborator positions on the current canvas.
// ## Parameters
// - `Child`: Content under the cursors.
// - `People`: Collaborator names, positions and presentation state.
// - `Viewport`: Current canvas transform.
// ## Returns
// The child with native cursors drawn over it.
public function View(Child: E.Element of string; People: array of W.Person; Viewport: N.View): E.Element of string;
begin
  return W.LiveCursors of string('collaborators', People, Child, new W.Options(View := Viewport));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Person.Name / X / Y` | Supply a stable name and the position in canvas coordinates. |
| `Says / Speaking / Away` | null / false / false. Away hides the cursor and dims the avatar in the presence pill. |
| `Tone / Color` | Neutral / null. Pass Color to override the theme tone. |
| `View / NoGlide` | null / false. With a null View, positions are relative to the child. NoGlide applies positions immediately. |

## Interaction

Render updated positions when the application receives them. Keep the
same canvas transform for the child and cursor overlay so they move and
zoom together.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Canvas/LiveCursors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Canvas/View.fpas)
- [Presence pill](presence-pill.md)
- [Canvas and bars](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [LiveCursors.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/LiveCursors.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.LiveCursors.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.LiveCursors.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiWindow.LiveCursors.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.LiveCursors.Types.cs)
  - [CuiWindow.LiveCursors.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.LiveCursors.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [live_cursors.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/live_cursors.c)
  - [live_cursors.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/live_cursors.h)
