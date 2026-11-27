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
`Rectangle`, `Border`, `Gradient`, `Shadow`, and `Image` accept a uniform
radius or `Painting.Corners` with a radius for each corner.
`Image` draws the whole texture by default. Pass `Painting.ImageRegion`
to draw a region. Zero through one spans each texture axis. A negative
span reverses that axis.

`FontScale()` is the user text multiplier; apply it to explicit text sizes when
desired. `Measure` uses the exact supplied font role and logical size, matching
`Text`. Both use the window's current face for that role. To draw or measure
part of a string, pass that part as a valid Unicode string.
`Time()` returns the cumulative native clock in seconds.
`DeltaTime()` returns the time since the preceding frame, capped at 0.25 seconds
for animation. Use `Frames.Redraw` for drawing driven by an application timer.
`Disabled()` reports whether the canvas was built inside a disabled scope,
so custom painting can follow the surrounding control state. `Focused()` reports
native focus ownership. Use `FocusVisible()` to decide when to draw a keyboard
focus ring.

Use `SetCursor` to request a cursor for the current frame. The last CUI request
takes priority, including paint requests after `Canvases.Options.HoverCursor`.
CUI starts the next frame with the arrow cursor.

CUI clips painting to the canvas. `PushClip` intersects that clip; `PopClip`
cannot escape the callback's scopes. Unclosed clips unwind on return or error.
If the native clip stack is full, `PushClip` raises an error and preserves the
current clip.
Do not retain a surface, use it from workers, reenter the window/session or
change application state while painting. Native painting can run repeatedly
without Update. Managed failures surface after CUI returns to presentation.

Images reuse CUI textures by invocation slot and immutable source identity.
Retain pixel arrays while displayed; replace the array when pixels change.
Replacing, omitting or removing a canvas releases its unused image resources;
window disposal releases the remainder. Image loading/drawing remains native.

`OnPointer` emits `Hovered`, the active press state `Pressed`, local X/Y and the
previously laid-out bounds. It emits on hover/press transitions and pointer
movement while active, without an idle message loop. Disabling a hovered canvas
emits one leave. Removing it retires its state. `Clicked` remains activation;
`OnWheel` remains an explicit opt-in to native two-axis wheel ownership.

With `OnWheel`, CUI routes wheel movement over the whole canvas by default.
Call `AddWheelRegion` during painting to replace that area with one or more
canvas-local rectangles. The active drawing clip bounds each rectangle.
CUI routes wheel movement using the regions painted in the previous frame.
All rectangles share the canvas's `OnWheel` callback.

Painting is GUI-only. It adds no native resource dependency to the TUI host.

Use `Surface.Color(Token)` to resolve a semantic color for the current frame.
The returned color includes any active appearance or accent transition.
Pass the color to drawing operations such as `Circle`.

## Spring animation

Call `Surface.Animate(Id, Target)` to move a numeric drawing value with a
0.4-second spring and zero bounce. Call
`Surface.AnimateSpring(Id, Target, Duration, Bounce)` to set a positive
duration in seconds and bounce from zero through one. The target and options
must be finite. The first call returns the target.

Use a stable, nonempty ID within a canvas. Sample each ID once per frame.
CUI requests frames while the value moves and returns the target immediately
under reduced motion. Very short springs can finish within a single frame.
Retain the target in the model and use the returned value for painting.

## See also

- [Canvas](../text-and-display/canvas.md)
- [Fonts and native captions](../host.md#fonts-and-native-captions)
- [Image sources and updates](../text-and-display/image.md#usage)
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
