using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DrawingCanvas operation.</summary>
    /// <param name="id">id for the native widget.</param>
    /// <param name="widthKind">widthKind for the native widget.</param>
    /// <param name="width">width for the native widget.</param>
    /// <param name="heightKind">heightKind for the native widget.</param>
    /// <param name="height">height for the native widget.</param>
    /// <param name="shapes">shapes for the native widget.</param>
    /// <param name="enabled">enabled for the native widget.</param>
    /// <param name="x">x for the native widget.</param>
    /// <param name="y">y for the native widget.</param>
    /// <param name="w">w for the native widget.</param>
    /// <param name="h">h for the native widget.</param>
    /// <param name="wheel">Whether to register the canvas as a native wheel region.</param>
    /// <param name="wheelX">Consumed horizontal wheel delta, in lines.</param>
    /// <param name="wheelY">Consumed vertical wheel delta, in lines.</param>
    /// <param name="paint">Optional deferred owner-thread painting after native layout.</param>
    /// <param name="onPointer">Optional changed-state notification after native canvas input.</param>
    /// <returns>The native widget result.</returns>
    public unsafe bool DrawingCanvas(string id, int widthKind, float width, int heightKind, float height, CuiDrawingShape[] shapes, bool enabled, out float x, out float y, out float w, out float h, bool wheel, out float wheelX, out float wheelY, Action<CuiCanvasPainter>? paint = null, Action<CuiCanvasPointer>? onPointer = null)
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
        string key = Stable(id);
        nint user = RetainCanvasPaint(paint);
        int flags;
        fixed (CuiDrawingShape* data = shapes)
            flags = NativeMethods.DrawingCanvas(context, key, widthKind, width, heightKind, height,
                data, shapes.Length, enabled ? 1 : 0, out x, out y, out w, out h,
                wheel ? 1 : 0, out wheelX, out wheelY, CanvasPaintFunction(user), user, onPointer is null ? 0 : 1);
        if ((flags & 2) != 0) onPointer!(new((flags & 4) != 0, (flags & 8) != 0, x, y, w, h));
        return (flags & 1) != 0;
    }
}
