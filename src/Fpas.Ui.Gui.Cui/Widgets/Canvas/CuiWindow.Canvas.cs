// [Documentation](../../../../docs/ui/gui/text-and-display/canvas.md)
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Submits a canvas for native layout, drawing and input.</summary>
    /// <param name="id">Stable, nonempty canvas identifier.</param>
    /// <param name="widthKind">Width mode. Content=0, fill=1, logical points=2, percentage=3.</param>
    /// <param name="width">Width in logical points or percentage. Ignored for content and fill modes.</param>
    /// <param name="heightKind">Height mode using the same encoding as <paramref name="widthKind"/>.</param>
    /// <param name="height">Height in logical points or percentage. Ignored for content and fill modes.</param>
    /// <param name="shapes">Canvas-local primitives in paint order.</param>
    /// <param name="enabled">Whether the canvas participates in pointer and keyboard input inside an enabled parent.</param>
    /// <param name="x">Pointer X relative to the previously laid-out canvas.</param>
    /// <param name="y">Pointer Y relative to the previously laid-out canvas.</param>
    /// <param name="w">Canvas width from the previous layout in logical points.</param>
    /// <param name="h">Canvas height from the previous layout in logical points.</param>
    /// <param name="wheel">Whether to register the canvas as a native wheel region.</param>
    /// <param name="wheelX">Consumed horizontal wheel delta, in lines.</param>
    /// <param name="wheelY">Consumed vertical wheel delta, in lines.</param>
    /// <param name="paint">Optional deferred owner-thread painting after native layout.</param>
    /// <param name="onPointer">Optional changed-state notification after native canvas input.</param>
    /// <param name="focusable">Include in native Tab order and allow keyboard activation.</param>
    /// <param name="focus">Request focus for this submission.</param>
    /// <param name="cursor">Hover cursor ordinal, or -1 for the default.</param>
    /// <param name="onInput">Receives ordered component input after an enabled canvas is submitted.</param>
    /// <param name="enabledDuringResize">Keep an enabled canvas eligible for focus during passive resize if it was previously eligible.</param>
    /// <returns>Whether the user activated this canvas by pointer or keyboard.</returns>
    public unsafe bool DrawingCanvas(string id, int widthKind, float width, int heightKind, float height, CuiDrawingShape[] shapes, bool enabled, out float x, out float y, out float w, out float h, bool wheel, out float wheelX, out float wheelY, Action<CuiCanvasPainter>? paint = null, Action<CuiCanvasPointer>? onPointer = null,
        bool focusable = false, bool focus = false, int cursor = -1, Action<CuiCanvasInput>? onInput = null, bool enabledDuringResize = false)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        ArgumentNullException.ThrowIfNull(shapes);
        foreach (var shape in shapes)
            if (shape.Kind is < 0 or > 2 || !float.IsFinite(shape.X) || !float.IsFinite(shape.Y) ||
                !float.IsFinite(shape.A) || !float.IsFinite(shape.B) || !float.IsFinite(shape.Radius) ||
                shape.Radius < 0 || shape.Kind == 0 && (shape.A < 0 || shape.B < 0))
                throw new ArgumentException("Canvas shapes need finite coordinates and nonnegative dimensions.", nameof(shapes));
        if (cursor is < -1 or > 6) throw new ArgumentOutOfRangeException(nameof(cursor));
        string key = Stable(id);
        nint user = RetainCanvasPaint(paint);
        bool submittingInput = !renderingResize;
        int inputState = renderingResize ? (enabled || enabledDuringResize ? 2 : 0) : (enabled ? 1 : 0);
        int flags;
        fixed (CuiDrawingShape* data = shapes)
            flags = NativeMethods.DrawingCanvas(context, key, widthKind, width, heightKind, height,
                data, shapes.Length, inputState, out x, out y, out w, out h,
                wheel && submittingInput ? 1 : 0, out wheelX, out wheelY, CanvasPaintFunction(user), user,
                onPointer is not null && submittingInput ? 1 : 0,
                focusable ? 1 : 0, onInput is null ? 0 : 1, focus && submittingInput ? 1 : 0, cursor);
        if (!submittingInput) return false;
        if ((flags & 16) != 0 && onInput is not null)
            canvasHandlers[NativeMethods.LastId(context)] = onInput;
        if ((flags & 2) != 0) onPointer!(new((flags & 4) != 0, (flags & 8) != 0, x, y, w, h));
        return (flags & 1) != 0;
    }
}
