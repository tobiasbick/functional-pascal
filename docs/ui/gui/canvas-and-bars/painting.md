# Custom canvas painting

`Canvases.Canvas` keeps its declarative `Drawing` commands and accepts an optional
`Paint` callback. CUI runs it after layout with the final canvas size, on the
window owner thread. The callback's `Painting.Surface` forwards drawing to CUI:
rounded rectangles/borders, gradients, shadows, circles, lines, polylines, convex
polygons, arcs, Unicode text, icons, emoji and immutable image sources.

Pass an owner-thread `procedure(Surface: Painting.Surface)` as the `Paint`
argument of the normal `Canvas` constructor. See the
[complete example](../../../../examples/ui/gui/painting/README.md).

Positions and explicit sizes are logical points. CUI handles display DPI.
`FontScale()` is the user text multiplier; apply it to explicit text sizes when
desired. `Measure` uses the exact supplied font role and logical size, matching
`Text`. `Time()` and `DeltaTime()` expose the native frame clock in seconds;
request another frame with `Frames.Redraw` when animation needs it.
`Disabled()` reports whether the canvas was built inside a disabled scope,
so custom painting can follow the surrounding control state. `Focused()` reports
native focus ownership. Use `FocusVisible()` to decide when to draw a keyboard
focus ring.

CUI clips painting to the canvas. `PushClip` intersects that clip; `PopClip`
cannot escape the callback's scopes. Unclosed clips unwind on return or error.
Do not retain a surface, use it from workers, reenter the window/session or
change application state while painting. Native painting can run repeatedly
without Update. Managed failures surface after CUI returns to presentation.

Images reuse CUI textures by invocation slot and immutable source identity.
Retain pixel arrays while displayed; replace the array when pixels change.
Replacing, omitting or removing a canvas releases its unused image resources;
window disposal releases the remainder. Image loading/drawing remains native.

`OnPointer` emits `Hovered`, the native press-edge `Pressed`, local X/Y and the
previously laid-out bounds. It emits on hover/press transitions and pointer
movement while active, without an idle message loop. Disabling a hovered canvas
emits one leave. Removing it retires its state. `Clicked` remains activation;
`OnWheel` remains an explicit opt-in to native two-axis wheel ownership.

Painting is GUI-only. It adds no native resource dependency to the TUI host.

## See also

- [Canvas](../text-and-display/canvas.md)
- [Canvas and bars](README.md)
- [CUI widget inventory](../../../cui-widget-inventory.md#cui_canvas)

## Source files

- `src/Fpas.Ui/Widgets/Canvas/`
  - [Painting.fpas](../../../../src/Fpas.Ui/Widgets/Canvas/Painting.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Canvas/`
  - [PaintSurface.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Canvas/PaintSurface.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Canvas/`
  - [NativeMethods.Painting.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Canvas/NativeMethods.Painting.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Canvas/`
  - [CuiCanvasPainter.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiCanvasPainter.cs)
  - [CuiWindow.Painting.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Canvas/CuiWindow.Painting.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [painting.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/painting.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/canvas/`
  - [painting.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/painting.c)
  - [painting.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/canvas/painting.h)
- `tests/Fixtures/gui-host/`
  - [DrawingResourcesContract.fpas](../../../../tests/Fixtures/gui-host/DrawingResourcesContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Display/`
  - [CuiCanvasPaintingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiCanvasPaintingTests.cs)
